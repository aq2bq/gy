//! list: node rows, or the write units when asked (proposal-v3 2). A node's
//! status comes from the model, so the view never compares status strings.
use super::open_or_closed;
use super::writers::writer;
use crate::model::{
    NeedState, Node, NodeData, NodeId, NodeKind, Relation, RequirementState, criterion_satisfied,
    criterion_unsatisfied, edges, need_state, question_open, reference, unwaited,
};
use crate::ops::local_time;
use crate::ops::repository::{Error, Repository, Result, Store};
use serde::Serialize;
use std::fmt;

/// What to keep. `actor` and `since` switch the output to write units.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct Filter {
    pub kind: Option<NodeKind>,
    pub status: Option<String>,
    pub targets: Option<NodeId>,
    pub grep: Option<String>,
    pub actor: Option<String>,
    pub since: Option<u64>,
}

/// One node as a row: id, ref, kind, status, title, scope, created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Row {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub title: String,
    pub scope: String,
    pub created: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub unwaited: bool,
}
impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match &self.reference {
            Some(reference) => format!("{} ({reference})", self.id),
            None => self.id.clone(),
        };
        let mut line = format!("{id} {} ", self.kind.name());
        if let Some(status) = &self.status {
            line.push_str(status);
            line.push(' ');
        }
        line.push_str(&self.title);
        write!(f, "{line} ({} {})", self.scope, local_time(&self.created))
    }
}

/// One write unit: the history entry that a `--actor` / `--since` listing
/// returns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct LogRow {
    pub seq: u64,
    pub at: u64,
    pub actor: String,
    /// The writer as a reader sees them: `<by> / <actor>` or the actor
    /// (n-d36d).
    pub who: String,
    pub node: String,
    pub what: String,
    pub why: String,
    pub source: String,
}
impl fmt::Display for LogRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.seq, self.at, self.who, self.node, self.what, self.why, self.source
        )
    }
}

/// Node rows by default; write units when `actor` or `since` is set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Listing {
    Nodes(Vec<Row>),
    History(Vec<LogRow>),
}
impl fmt::Display for Listing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nodes(rows) => rows.iter().try_for_each(|row| writeln!(f, "{row}")),
            Self::History(rows) => rows.iter().try_for_each(|row| writeln!(f, "{row}")),
        }
    }
}

pub fn list<S: Store>(repo: &Repository<S>, filter: &Filter) -> Result<Listing> {
    if filter.actor.is_some() || filter.since.is_some() {
        return Ok(Listing::History(history(repo, filter)));
    }
    let status = match &filter.status {
        Some(text) => Some(parse_status(text)?),
        None => None,
    };
    let all = repo.all()?;
    let mut rows = Vec::new();
    for node in &all {
        if keep(node, filter, status, &all) {
            rows.push(row(node, &all));
        }
    }
    Ok(Listing::Nodes(rows))
}

/// The history as write units, filtered by actor and by the transaction
/// sequence (strictly after `since`). Every entry of one transaction shares
/// the sequence.
fn history<S: Store>(repo: &Repository<S>, filter: &Filter) -> Vec<LogRow> {
    let mut rows = Vec::new();
    for entry in repo.store().history() {
        if filter.since.is_some_and(|since| entry.seq <= since) {
            continue;
        }
        if filter.actor.as_deref().is_some_and(|wanted| {
            entry.actor.name() != wanted && entry.by.as_deref() != Some(wanted)
        }) {
            continue;
        }
        let actor = entry.actor.name().to_string();
        rows.push(LogRow {
            seq: entry.seq,
            at: entry.at,
            who: writer(entry.by.as_deref(), &actor),
            actor,
            node: entry.node.clone(),
            what: entry.what.clone(),
            why: entry.why.clone(),
            source: entry.source.clone(),
        });
    }
    rows
}

fn keep(node: &Node, filter: &Filter, status: Option<Status>, all: &[Node]) -> bool {
    if filter.kind.is_some_and(|kind| node.kind() != kind) {
        return false;
    }
    if status.is_some_and(|status| !matches_status(node, status, all)) {
        return false;
    }
    if let Some(target) = &filter.targets {
        if !edges(node, Relation::Targets).contains(target) {
            return false;
        }
    }
    if let Some(grep) = &filter.grep {
        if matches(node, grep).is_none() {
            return false;
        }
    }
    true
}

/// Where a query hit a node. Ordered so a caller can sort by it: an exact id
/// or alias first, then a name (or title) hit, then a body-only hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Match {
    /// The id or an alias is the query itself.
    Exact,
    /// The id, an alias, or the title contains the query.
    Title,
    /// Only the body contains the query.
    Body,
}

/// The one place the ledger judges whether a query hits a node (d-6a45): the
/// id, every alias, the title, and the body, each as a case-insensitive
/// substring. The grade says where it hit, so `list --grep` and the Web search
/// read the same rule.
pub fn matches(node: &Node, query: &str) -> Option<Match> {
    let query = query.to_lowercase();
    if query.is_empty() {
        return None;
    }
    let id = node.id().to_string().to_lowercase();
    let aliases: Vec<String> = node
        .aliases()
        .iter()
        .map(|alias| alias.0.to_lowercase())
        .collect();
    let title = node.title().to_lowercase();
    if id == query || aliases.iter().any(|alias| alias == &query) {
        return Some(Match::Exact);
    }
    if id.contains(&query) || title.contains(&query) || aliases.iter().any(|a| a.contains(&query)) {
        return Some(Match::Title);
    }
    if node.body().to_lowercase().contains(&query) {
        return Some(Match::Body);
    }
    None
}

fn row(node: &Node, all: &[Node]) -> Row {
    Row {
        id: node.id().to_string(),
        reference: reference(node).map(str::to_string),
        kind: node.kind(),
        status: status_of(node, all),
        title: node.title().to_string(),
        scope: node.scope().to_string(),
        created: node.created().to_string(),
        unwaited: unwaited(node, all),
    }
}

/// The status a row prints: the model's own name, and for a need the derived
/// state from the graph (n-35cf). Shared with the ego map's boxes, so the two
/// views never drift (n-dc1a).
pub(crate) fn status_of(node: &Node, all: &[Node]) -> Option<String> {
    match node.data() {
        NodeData::Need(_) => Some(need_state(node, all).name().to_string()),
        NodeData::Question(_) => Some(open_or_closed(!question_open(node.data())).to_string()),
        NodeData::Requirement(data) => Some(data.state.name().to_string()),
        NodeData::Criterion(_) => Some(
            if criterion_satisfied(node.data()) {
                "satisfied"
            } else {
                "unsatisfied"
            }
            .to_string(),
        ),
        NodeData::Decision(_) => None,
    }
}

/// A parsed `--status`: the one place a status name becomes a model value.
#[derive(Debug, Clone, Copy)]
enum Status {
    Open,
    Closed,
    Done,
    Satisfied,
    Unsatisfied,
    Requirement(RequirementState),
}

/// The status words a filter takes, whatever their case (n-a56f).
const STATUS_WORDS: [&str; 8] = [
    "open",
    "closed",
    "done",
    "satisfied",
    "unsatisfied",
    "filed",
    "approved",
    "cancelled",
];

fn parse_status(text: &str) -> Result<Status> {
    let states = [
        RequirementState::Filed,
        RequirementState::Approved,
        RequirementState::Done,
        RequirementState::Cancelled,
    ];
    let wanted = text.to_lowercase();
    match wanted.as_str() {
        "open" => Ok(Status::Open),
        "closed" => Ok(Status::Closed),
        "done" => Ok(Status::Done),
        "satisfied" => Ok(Status::Satisfied),
        "unsatisfied" => Ok(Status::Unsatisfied),
        _ => states
            .into_iter()
            .find(|state| state.name() == wanted)
            .map(Status::Requirement)
            .ok_or_else(|| {
                Error::invalid(format!(
                    "unknown status {text}; expected one of {}",
                    STATUS_WORDS.join(", ")
                ))
            }),
    }
}

fn matches_status(node: &Node, wanted: Status, all: &[Node]) -> bool {
    match (node.data(), wanted) {
        (NodeData::Need(_), Status::Open) => need_state(node, all) == NeedState::Open,
        (NodeData::Need(_), Status::Closed) => need_state(node, all) == NeedState::Closed,
        (NodeData::Need(_), Status::Done) => need_state(node, all) == NeedState::Done,
        (NodeData::Question(_), Status::Open) => question_open(node.data()),
        (NodeData::Question(_), Status::Closed) => !question_open(node.data()),
        (NodeData::Requirement(data), Status::Done) => data.state == RequirementState::Done,
        (NodeData::Requirement(data), Status::Requirement(state)) => data.state == state,
        (NodeData::Criterion(_), Status::Satisfied) => criterion_satisfied(node.data()),
        (NodeData::Criterion(_), Status::Unsatisfied) => criterion_unsatisfied(node.data()),
        _ => false,
    }
}
