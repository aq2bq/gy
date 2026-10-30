//! One read of the ledger for the three reading routes (d-6d90): the nodes, the
//! log, and the indexes over them. A route builds this once, so nothing it asks
//! is scanned twice and no two routes derive the same thing differently — the
//! judgements themselves stay in gy-ledger (`model/derive.rs`).
use chrono::{DateTime, Datelike, Days, NaiveDate, SecondsFormat};
use gy_ledger::{
    HistoryEntry, NeedState, Node, NodeData, NodeKind, Relation, Repository, Result, Store,
    need_state, status_of,
};
use std::collections::{HashMap, HashSet};

/// An edge as it arrives at a node: the node it leaves, the relation, and the
/// passage the edge names.
pub struct Arrival {
    pub from: usize,
    pub relation: Relation,
    pub mark: Option<String>,
}

/// What a node did, as the crossing screen counts it (d-6d90).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Decided,
    QuestionClosed,
    QuestionRaised,
    RequirementApproved,
    RequirementDone,
    CriterionSatisfied,
    NeedClosed,
}
impl Act {
    pub fn name(self) -> &'static str {
        match self {
            Self::Decided => "decided",
            Self::QuestionClosed => "question-closed",
            Self::QuestionRaised => "question-raised",
            Self::RequirementApproved => "requirement-approved",
            Self::RequirementDone => "requirement-done",
            Self::CriterionSatisfied => "criterion-satisfied",
            Self::NeedClosed => "need-closed",
        }
    }
    /// Every act, in the order an answer lists them.
    pub const ALL: [Self; 7] = [
        Self::Decided,
        Self::QuestionClosed,
        Self::QuestionRaised,
        Self::RequirementApproved,
        Self::RequirementDone,
        Self::CriterionSatisfied,
        Self::NeedClosed,
    ];
}

/// One instant a node moved: a birth (`act` is `None`) or a counted act.
#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub at: u64,
    pub act: Option<Act>,
}

pub struct Snapshot<'a> {
    nodes: Vec<Node>,
    index: HashMap<String, usize>,
    /// The edges into each node, by the index of the node they leave.
    incoming: Vec<Vec<Arrival>>,
    /// The instant of the newest write to each node the log still names.
    last_write: HashMap<String, u64>,
    history: &'a [HistoryEntry],
    /// The last write of the point this snapshot reads.
    pub seq: u64,
    pub at: u64,
}

impl<'a> Snapshot<'a> {
    /// Read the ledger once: the nodes, the log, and the reverse edges.
    pub fn read<S: Store>(repo: &'a Repository<S>) -> Result<Self> {
        let nodes = repo.all()?;
        let history = repo.store().history();
        let index: HashMap<String, usize> = nodes
            .iter()
            .enumerate()
            .map(|(at, node)| (node.id().to_string(), at))
            .collect();
        let mut incoming: Vec<Vec<Arrival>> = (0..nodes.len()).map(|_| Vec::new()).collect();
        for (from, node) in nodes.iter().enumerate() {
            for edge in node.links() {
                let Some(&to) = index.get(&edge.to.to_string()) else {
                    continue;
                };
                incoming[to].push(Arrival {
                    from,
                    relation: edge.label,
                    mark: edge.mark.clone(),
                });
            }
        }
        let (mut seq, mut at) = (0, 0);
        let mut last_write = HashMap::new();
        for entry in history {
            seq = entry.seq;
            at = entry.at;
            if !entry.node.is_empty() {
                last_write.insert(entry.node.clone(), entry.at);
            }
        }
        Ok(Self {
            nodes,
            index,
            incoming,
            last_write,
            history,
            seq,
            at,
        })
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.index.get(id).copied()
    }

    pub fn node(&self, at: usize) -> &Node {
        &self.nodes[at]
    }

    pub fn arrivals(&self, at: usize) -> &[Arrival] {
        &self.incoming[at]
    }

    /// The instant of the newest write to a node, or `None` for a node the log
    /// does not name (a seed put in without a transaction).
    pub fn last_write_of(&self, at: usize) -> Option<u64> {
        self.last_write
            .get(&self.nodes[at].id().to_string())
            .copied()
    }

    /// The word the list view prints, and for a decision the one the graph
    /// gives it: another decision's `supersedes` points at it or not.
    pub fn state(&self, at: usize) -> Option<String> {
        let node = &self.nodes[at];
        match node.data() {
            NodeData::Decision(_) => Some(
                if self.superseded(at) {
                    "superseded"
                } else {
                    "live"
                }
                .to_string(),
            ),
            _ => status_of(node, &self.nodes),
        }
    }

    /// Whether another decision supersedes this one.
    pub fn superseded(&self, at: usize) -> bool {
        self.incoming[at]
            .iter()
            .any(|arrival| arrival.relation == Relation::Supersedes)
    }

    /// A node a screen shows as still going: the words an open row carries.
    pub fn is_open(&self, at: usize) -> bool {
        matches!(
            self.state(at).as_deref(),
            Some("open" | "filed" | "approved" | "unsatisfied" | "live")
        )
    }

    /// The nodes this one points at by one relation.
    pub fn pointing_at(&self, at: usize, relation: Relation) -> Vec<usize> {
        self.nodes[at]
            .links()
            .iter()
            .filter(|edge| edge.label == relation)
            .filter_map(|edge| self.index_of(&edge.to.to_string()))
            .collect()
    }

    /// Every instant this node moved, newest stamp last. A birth carries no act
    /// unless the model counts it (a decision and a question are born as
    /// themselves); an edit-only write is not here at all.
    pub fn moves(&self, at: usize) -> Vec<Move> {
        let node = &self.nodes[at];
        let mut moves = Vec::new();
        if let Some(born) = instant(node.created()) {
            let act = match node.kind() {
                NodeKind::Decision => Some(Act::Decided),
                NodeKind::Question => Some(Act::QuestionRaised),
                _ => None,
            };
            moves.push(Move { at: born, act });
        }
        let closed_at = self.last_write_of(at);
        match node.data() {
            NodeData::Question(data) if data.closure.is_some() => {
                if let Some(held) = closed_at {
                    moves.push(Move {
                        at: held,
                        act: Some(Act::QuestionClosed),
                    });
                }
            }
            NodeData::Requirement(data) => {
                let mut add = |stamp: Option<&String>, act: Option<Act>| {
                    if let Some(at) = stamp.and_then(|text| instant(text)) {
                        moves.push(Move { at, act });
                    }
                };
                add(
                    data.approval.as_ref().map(|held| &held.at),
                    Some(Act::RequirementApproved),
                );
                add(
                    data.completion.as_ref().map(|held| &held.at),
                    Some(Act::RequirementDone),
                );
                add(data.cancellation.as_ref().map(|held| &held.at), None);
            }
            NodeData::Criterion(data) if data.satisfied => {
                if let Some(at) = data.satisfied_at.as_ref().and_then(|text| instant(text)) {
                    moves.push(Move {
                        at,
                        act: Some(Act::CriterionSatisfied),
                    });
                }
            }
            NodeData::Need(data) if data.closed.is_some() => {
                if let Some(held) = closed_at {
                    moves.push(Move {
                        at: held,
                        act: Some(Act::NeedClosed),
                    });
                }
            }
            _ => {}
        }
        moves
    }

    /// When a need reached `done` or was closed by hand: the newest of its
    /// requirements' completion or cancellation and its criteria's satisfied
    /// instants, falling back to the newest write to the need itself.
    pub fn need_done_at(&self, at: usize, state: NeedState) -> Option<u64> {
        if state == NeedState::Open {
            return None;
        }
        let node = &self.nodes[at];
        let mut done = self.last_write_of(at).or_else(|| instant(node.created()));
        for requirement in self.pointing_at(at, Relation::FiledAs) {
            if let NodeData::Requirement(data) = self.nodes[requirement].data() {
                for stamp in [
                    data.completion.as_ref().map(|held| &held.at),
                    data.cancellation.as_ref().map(|held| &held.at),
                ]
                .into_iter()
                .flatten()
                {
                    done = done.max(instant(stamp));
                }
            }
        }
        for criterion in self.pointing_at(at, Relation::Targets) {
            if let NodeData::Criterion(data) = self.nodes[criterion].data() {
                done = done.max(data.satisfied_at.as_deref().and_then(instant));
            }
        }
        done
    }

    /// Every Monday-start week the ledger spans: from the week of the earliest
    /// instant it holds (a write or a created) through the week of the latest.
    pub fn weeks(&self) -> Vec<NaiveDate> {
        let mut first: Option<u64> = None;
        let mut last: Option<u64> = None;
        let mut hold = |at: u64| {
            first = Some(first.map_or(at, |held| held.min(at)));
            last = Some(last.map_or(at, |held| held.max(at)));
        };
        for entry in self.history {
            hold(entry.at);
        }
        for node in &self.nodes {
            if let Some(at) = instant(node.created()) {
                hold(at);
            }
        }
        match (first, last) {
            (Some(first), Some(last)) => {
                weeks_between(week_start(day_of(first)), week_start(day_of(last)))
            }
            _ => Vec::new(),
        }
    }

    /// Writes per scope and day, counted once per write for each scope it
    /// touched, over the `days` days ending on the answer's last write. The
    /// start day comes back with it; a day without a write has no entry.
    pub fn pulse(&self, days: u64) -> (NaiveDate, HashMap<(String, i64), usize>) {
        let end = day_of(self.at);
        let start = end - Days::new(days.saturating_sub(1));
        let mut counts: HashMap<(String, i64), usize> = HashMap::new();
        let mut counted: HashSet<(u64, String)> = HashSet::new();
        for entry in self.history {
            if entry.node.is_empty() {
                continue;
            }
            let Some(at) = self.index_of(&entry.node) else {
                continue;
            };
            let day = day_of(entry.at);
            if day < start || day > end {
                continue;
            }
            let scope = self.nodes[at].scope().to_string();
            if counted.insert((entry.seq, scope.clone())) {
                *counts.entry((scope, (day - start).num_days())).or_default() += 1;
            }
        }
        (start, counts)
    }

    /// Whether a need carries a wait now: it waits on an open question or an
    /// in-progress requirement, or depends on an open need.
    pub fn waiting_on(&self, at: usize) -> Vec<usize> {
        let mut waiting: Vec<usize> = self
            .pointing_at(at, Relation::WaitsOn)
            .into_iter()
            .filter(|held| self.is_open(*held))
            .collect();
        waiting.extend(
            self.pointing_at(at, Relation::DependsOn)
                .into_iter()
                .filter(|held| need_state(&self.nodes[*held], &self.nodes) == NeedState::Open),
        );
        waiting
    }
}

/// An instant as epoch seconds, from the `YYYY-MM-DDTHH:MM:SSZ` form the model
/// stores. Text that is not one is `None`.
pub fn instant(text: &str) -> Option<u64> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|at| at.timestamp().max(0) as u64)
}

/// An instant as the model writes one: `YYYY-MM-DDTHH:MM:SSZ`.
pub fn rfc3339(at: u64) -> String {
    DateTime::from_timestamp(at as i64, 0)
        .map(|when| when.to_rfc3339_opts(SecondsFormat::Secs, true))
        .unwrap_or_default()
}

/// The UTC day an instant falls on.
pub fn day_of(at: u64) -> NaiveDate {
    DateTime::from_timestamp(at as i64, 0)
        .map(|when| when.date_naive())
        .unwrap_or_default()
}

/// The Monday of the week a day is in.
pub fn week_start(day: NaiveDate) -> NaiveDate {
    day - Days::new(u64::from(day.weekday().num_days_from_monday()))
}

/// Every Monday-start week from `first` through `last`, both included.
pub fn weeks_between(first: NaiveDate, last: NaiveDate) -> Vec<NaiveDate> {
    let mut weeks = Vec::new();
    let mut week = first;
    while week <= last {
        weeks.push(week);
        week = week + Days::new(7);
    }
    weeks
}
