//! `GET /api/overview`: what the first screen asks, the shape of the progress
//! (d-e239, d-6d90). It reads a ledger whole, judges with gy-ledger's own
//! derivations (`model/derive.rs`), and places nothing.
//!
//! # The answer
//!
//! Instants are RFC 3339 UTC (`2026-10-30T04:46:44Z`), days are
//! `YYYY-MM-DD`, and `seq`/`at` name the point in the log the answer reads,
//! as in `/api/now`.
//!
//! ```json
//! {
//!   "seq": 4853,
//!   "at": "2026-10-30T04:46:44Z",
//!   "counts": {
//!     "open_needs": 48,
//!     "blocked_needs": 20,
//!     "open_questions": 45,
//!     "unmet_criteria": 62,
//!     "closed_needs": 218
//!   },
//!   "weeks": [
//!     {"from": "2026-09-07", "opened": 12, "done": 7}
//!   ],
//!   "scopes": [
//!     {
//!       "name": "crm",
//!       "open": 3,
//!       "closed": 40,
//!       "blocked": 1,
//!       "pulse": [{"day": "2026-10-01", "writes": 4}]
//!     }
//!   ],
//!   "needs": [
//!     {
//!       "id": "n-65c9",
//!       "scope": "crm",
//!       "title": "…",
//!       "stage": "before",
//!       "blocked": true,
//!       "last_moved": "2026-10-12T03:00:00Z",
//!       "requirements": {"filed": 1, "approved": 0, "done": 2, "cancelled": 0},
//!       "criteria": {"satisfied": 1, "unsatisfied": 1, "withdrawn": 0},
//!       "waiting": [
//!         {"id": "q-164b", "kind": "Question", "title": "…", "decider": "マスター"}
//!       ]
//!     }
//!   ],
//!   "blockers": [
//!     {
//!       "id": "q-164b",
//!       "kind": "Question",
//!       "scope": "crm",
//!       "title": "…",
//!       "decider": "マスター",
//!       "needs": ["n-b0f5", "n-da7f"],
//!       "via": ["n-65c9"]
//!     }
//!   ],
//!   "deciders": [{"name": "マスター", "count": 34}],
//!   "overdue": [
//!     {"id": "ac-1a2b", "scope": "crm", "title": "…", "due": "2026-10-01",
//!      "responsible": "…", "then": "…"}
//!   ],
//!   "unheld": [
//!     {"id": "q-164b", "kind": "Question", "scope": "crm", "state": "open", "title": "…"}
//!   ]
//! }
//! ```
//!
//! # The keys
//!
//! - `counts`: `open_needs` (a need the graph derives `open`), `blocked_needs`
//!   (an open need with at least one open wait), `open_questions` (a question
//!   still open), `unmet_criteria` (a criterion derived `unsatisfied`; a
//!   withdrawn one is not counted), `closed_needs` (every need that is not
//!   open: derived `done` or closed by hand).
//! - `weeks`: Monday-start weeks, from the week of the earliest instant the
//!   ledger holds (a write or a node's `created`) through the week of the
//!   latest. Every week of that span is present, zeros included. `opened`
//!   counts the needs created in the week; `done` the needs that reached
//!   `done` or were closed in it (see "When a need moved").
//! - `scopes`: every scope the ledger holds, the one with the most open needs
//!   first, ties by name. `open`/`closed`/`blocked` count its needs as
//!   `counts` does. `pulse` is its writes: one entry per day that has any, in
//!   the 49 days ending on the day of the answer's last write, counted once per
//!   write for each scope that write touched. Days without a write are left
//!   out; the screen draws the zeros.
//! - `needs`: every open need, needs with a wait first, then the one that moved
//!   last, newest first, ties by id. `stage` is `before` (着手前: no
//!   requirement yet), `build` (作っている: a requirement is filed or
//!   approved), `verify` (確かめ待ち: a requirement is done, none is still in
//!   progress) or `close` (閉じ待ち: its criteria are all satisfied, so no
//!   requirement is left to finish it). A need that is not open is not here.
//!   `blocked` is `true` when `waiting` is not empty. `last_moved` is the
//!   newest write to the need, its filed requirements, its targeted criteria,
//!   the things it waits on, or any node pointing at it. `requirements` counts
//!   the requirements it filed, by their state, and `criteria` counts the
//!   criteria it targets, each as the model derives it (`satisfied`,
//!   `unsatisfied`, `withdrawn`). `waiting` is what holds it: the open questions
//!   and requirements it `waits-on` and the open needs it `depends-on`, each
//!   `{id, kind, title, decider?}` — `decider` is present only for a question.
//! - `blockers`: the roots of the waits. A root is a waiting question, a
//!   waiting requirement, or a need that waits on nothing while another need
//!   depends on it; a need blocked only through the needs it depends on is not
//!   a root but a `via` step. One entry per root, the root blocking the most
//!   needs first, ties by id. `needs` is every open need this root holds;
//!   `via` is the chain of needs between those needs and the root, outermost
//!   first, empty when the root holds a need directly. `decider` is present
//!   only for a question.
//! - `deciders`: the open questions by the person who decides them, most first,
//!   ties by name. The name is the record's own, so the spellings it mixes
//!   (`マスター`, `master`) stay apart; a question that names none counts under
//!   `""`.
//! - `overdue`: the criteria whose `due` has come and that are still
//!   `unsatisfied` — the same wait `handover` dates (r-e9ca) — earliest date
//!   first, ties by id. `due`, `responsible`, and `then` are the node's own
//!   words. A withdrawn criterion is not overdue: nobody waits for it any more.
//! - `unheld`: what nobody is working on, questions first then criteria, newest
//!   first within each kind, ties by id. That is every open question no node
//!   `waits-on` and no node `raised` (gy-ledger's `unwaited`), and every
//!   `unsatisfied` criterion that no open need and no in-progress requirement
//!   targets. A withdrawn criterion is not here either.
//!
//! # When a need moved
//!
//! The model stamps a need's close with its `by` and evidence but no instant,
//! and a question's closure the same way, so those two take the instant of the
//! newest write to the node itself. A requirement's approval, completion and
//! cancellation carry their own instants, and so does a criterion's
//! `satisfied_at`. A need derived `done` moved when the last of those
//! happened, or, when the record carries none of them, at the newest write to
//! the need. Reads the model already records are the only source: an edit that
//! changes nothing a reader sees still counts as a write to `last_moved`, but
//! it never becomes an event of its own.
use crate::http::Response;
use crate::snapshot::{self, Snapshot, day_of, instant};
use gy_ledger::{
    CriterionState, NeedState, Node, NodeData, NodeKind, Relation, Repository, RequirementState,
    Store, criterion_state, criterion_wait, need_state, question_open, unwaited,
};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

/// The days a scope's pulse covers.
const PULSE_DAYS: u64 = 49;

/// The counts the first screen puts at the top.
#[derive(Serialize)]
struct Counts {
    open_needs: usize,
    blocked_needs: usize,
    open_questions: usize,
    unmet_criteria: usize,
    closed_needs: usize,
}

#[derive(Serialize)]
struct Week {
    from: String,
    opened: usize,
    done: usize,
}

#[derive(Serialize)]
struct PulseDay {
    day: String,
    writes: usize,
}

#[derive(Serialize)]
struct ScopeLine {
    name: String,
    open: usize,
    closed: usize,
    blocked: usize,
    pulse: Vec<PulseDay>,
}

#[derive(Serialize)]
struct Waiting {
    id: String,
    kind: NodeKind,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    decider: Option<String>,
}

#[derive(Serialize, Default)]
struct RequirementCounts {
    filed: usize,
    approved: usize,
    done: usize,
    cancelled: usize,
}

#[derive(Serialize, Default)]
struct CriterionCounts {
    satisfied: usize,
    unsatisfied: usize,
    withdrawn: usize,
}

#[derive(Serialize)]
struct NeedLine {
    id: String,
    scope: String,
    title: String,
    stage: &'static str,
    blocked: bool,
    last_moved: String,
    requirements: RequirementCounts,
    criteria: CriterionCounts,
    waiting: Vec<Waiting>,
}

#[derive(Serialize)]
struct Blocker {
    id: String,
    kind: NodeKind,
    scope: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    decider: Option<String>,
    needs: Vec<String>,
    via: Vec<String>,
}

#[derive(Serialize)]
struct Decider {
    name: String,
    count: usize,
}

#[derive(Serialize)]
struct Overdue {
    id: String,
    scope: String,
    title: String,
    due: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    responsible: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    then: Option<String>,
}

#[derive(Serialize)]
struct Unheld {
    id: String,
    kind: NodeKind,
    scope: String,
    state: String,
    title: String,
}

#[derive(Serialize)]
struct Overview {
    seq: u64,
    at: String,
    counts: Counts,
    weeks: Vec<Week>,
    scopes: Vec<ScopeLine>,
    needs: Vec<NeedLine>,
    blockers: Vec<Blocker>,
    deciders: Vec<Decider>,
    overdue: Vec<Overdue>,
    unheld: Vec<Unheld>,
}

/// What the first screen needs to know about one need.
struct Info {
    at: usize,
    state: NeedState,
    stage: &'static str,
    waiting: Vec<usize>,
    last_moved: u64,
    done_at: Option<u64>,
    requirements: RequirementCounts,
    criteria: CriterionCounts,
}

pub fn answer<S: Store>(repo: &Repository<S>) -> Response {
    let snapshot = match Snapshot::read(repo) {
        Ok(snapshot) => snapshot,
        Err(error) => return Response::text(500, &error.to_string()),
    };
    Response::json(&view(&snapshot))
}

/// The whole answer, from one read.
fn view(snap: &Snapshot) -> Overview {
    let infos = infos(snap);
    let needs: Vec<&Info> = infos.iter().flatten().collect();
    let open: Vec<&Info> = needs
        .iter()
        .copied()
        .filter(|info| info.state == NeedState::Open)
        .collect();
    let blocked: Vec<&Info> = open
        .iter()
        .copied()
        .filter(|i| !i.waiting.is_empty())
        .collect();
    Overview {
        seq: snap.seq,
        at: snapshot::rfc3339(snap.at),
        counts: Counts {
            open_needs: open.len(),
            blocked_needs: blocked.len(),
            open_questions: snap
                .nodes()
                .iter()
                .filter(|node| question_open(node.data()))
                .count(),
            unmet_criteria: snap
                .nodes()
                .iter()
                .filter(|node| node.kind() == NodeKind::Criterion)
                .filter(|node| criterion_state(node, snap.nodes()) == CriterionState::Unsatisfied)
                .count(),
            closed_needs: needs.len() - open.len(),
        },
        weeks: weeks(snap, &needs),
        scopes: scopes(snap, &needs),
        needs: need_lines(snap, &open),
        blockers: blockers(snap, &infos, &open),
        deciders: deciders(snap),
        overdue: overdue(snap),
        unheld: unheld(snap),
    }
}

/// One pass over the needs, so no two parts ask the graph the same thing.
fn infos(snap: &Snapshot) -> Vec<Option<Info>> {
    (0..snap.nodes().len())
        .map(|at| (snap.node(at).kind() == NodeKind::Need).then(|| info(snap, at)))
        .collect()
}

fn info(snap: &Snapshot, at: usize) -> Info {
    let node = snap.node(at);
    let state = need_state(node, snap.nodes());
    let requirements: Vec<usize> = snap.pointing_at(at, Relation::FiledAs);
    let criteria: Vec<usize> = snap.pointing_at(at, Relation::Targets);
    let waiting = snap.waiting_on(at);
    Info {
        at,
        state,
        stage: stage(snap, &requirements, &criteria, state),
        waiting,
        last_moved: last_moved(snap, at, &requirements, &criteria),
        done_at: snap.need_done_at(at, state),
        requirements: requirement_counts(snap, &requirements),
        criteria: criterion_counts(snap, &criteria),
    }
}

/// Where a need stands, from the same edges the board draws.
fn stage(
    snap: &Snapshot,
    requirements: &[usize],
    criteria: &[usize],
    state: NeedState,
) -> &'static str {
    if state != NeedState::Open {
        return "done";
    }
    let live = requirements.iter().any(|at| {
        matches!(
            snap.node(*at).state(),
            Some(RequirementState::Filed | RequirementState::Approved)
        )
    });
    let finished = requirements
        .iter()
        .any(|at| snap.node(*at).state() == Some(RequirementState::Done));
    let met = !criteria.is_empty()
        && criteria
            .iter()
            .all(|at| gy_ledger::criterion_satisfied(snap.node(*at).data()));
    if live {
        "build"
    } else if finished {
        "verify"
    } else if met {
        "close"
    } else {
        "before"
    }
}

/// The newest write to the need or to what it is made of.
fn last_moved(snap: &Snapshot, at: usize, requirements: &[usize], criteria: &[usize]) -> u64 {
    let mut related: Vec<usize> = vec![at];
    related.extend_from_slice(requirements);
    related.extend_from_slice(criteria);
    related.extend(snap.pointing_at(at, Relation::WaitsOn));
    related.extend(snap.arrivals(at).iter().map(|arrival| arrival.from));
    related
        .into_iter()
        .filter_map(|held| snap.last_write_of(held))
        .max()
        .or_else(|| instant(snap.node(at).created()))
        .unwrap_or(0)
}

fn requirement_counts(snap: &Snapshot, requirements: &[usize]) -> RequirementCounts {
    let mut counts = RequirementCounts::default();
    for at in requirements {
        match snap.node(*at).state() {
            Some(RequirementState::Filed) => counts.filed += 1,
            Some(RequirementState::Approved) => counts.approved += 1,
            Some(RequirementState::Done) => counts.done += 1,
            Some(RequirementState::Cancelled) => counts.cancelled += 1,
            None => {}
        }
    }
    counts
}

fn criterion_counts(snap: &Snapshot, criteria: &[usize]) -> CriterionCounts {
    let mut counts = CriterionCounts::default();
    for at in criteria {
        match criterion_state(snap.node(*at), snap.nodes()) {
            CriterionState::Satisfied => counts.satisfied += 1,
            CriterionState::Unsatisfied => counts.unsatisfied += 1,
            CriterionState::Withdrawn => counts.withdrawn += 1,
        }
    }
    counts
}

/// The open needs' cards, the ones that wait first, then the newest move.
fn need_lines(snap: &Snapshot, open: &[&Info]) -> Vec<NeedLine> {
    let mut ordered: Vec<&&Info> = open.iter().collect();
    ordered.sort_by(|left, right| {
        left.waiting
            .is_empty()
            .cmp(&right.waiting.is_empty())
            .then_with(|| right.last_moved.cmp(&left.last_moved))
            .then_with(|| {
                snap.node(left.at)
                    .id()
                    .to_string()
                    .cmp(&snap.node(right.at).id().to_string())
            })
    });
    ordered
        .into_iter()
        .map(|info| NeedLine {
            id: id(snap, info.at),
            scope: snap.node(info.at).scope().to_string(),
            title: snap.node(info.at).title().to_string(),
            stage: info.stage,
            blocked: !info.waiting.is_empty(),
            last_moved: snapshot::rfc3339(info.last_moved),
            requirements: RequirementCounts {
                filed: info.requirements.filed,
                approved: info.requirements.approved,
                done: info.requirements.done,
                cancelled: info.requirements.cancelled,
            },
            criteria: CriterionCounts {
                satisfied: info.criteria.satisfied,
                unsatisfied: info.criteria.unsatisfied,
                withdrawn: info.criteria.withdrawn,
            },
            waiting: info.waiting.iter().map(|at| waiting(snap, *at)).collect(),
        })
        .collect()
}

fn waiting(snap: &Snapshot, at: usize) -> Waiting {
    let node = snap.node(at);
    Waiting {
        id: id(snap, at),
        kind: node.kind(),
        title: node.title().to_string(),
        decider: decider(node),
    }
}

/// A question's decider, as the record writes it; no other kind has one.
fn decider(node: &Node) -> Option<String> {
    match node.data() {
        NodeData::Question(data) => data.decider.clone(),
        _ => None,
    }
}

/// The roots of every open need's wait, each with the needs it holds.
fn blockers(snap: &Snapshot, infos: &[Option<Info>], open: &[&Info]) -> Vec<Blocker> {
    let mut held: HashMap<usize, Held> = HashMap::new();
    for info in open {
        for (root, via) in roots(infos, info.at) {
            let entry = held.entry(root).or_insert_with(|| Held {
                needs: Vec::new(),
                via: Vec::new(),
            });
            let need = id(snap, info.at);
            if !entry.needs.contains(&need) {
                entry.needs.push(need);
            }
            if via.len() > entry.via.len() {
                entry.via = via;
            }
        }
    }
    let mut roots: Vec<(usize, Held)> = held.into_iter().collect();
    roots.sort_by(|left, right| {
        right
            .1
            .needs
            .len()
            .cmp(&left.1.needs.len())
            .then_with(|| id(snap, left.0).cmp(&id(snap, right.0)))
    });
    roots
        .into_iter()
        .map(|(root, held)| {
            let node = snap.node(root);
            Blocker {
                id: id(snap, root),
                kind: node.kind(),
                scope: node.scope().to_string(),
                title: node.title().to_string(),
                decider: decider(node),
                needs: held.needs,
                via: held.via.into_iter().map(|at| id(snap, at)).collect(),
            }
        })
        .collect()
}

/// What one root holds: the needs waiting on it, and the longest chain of needs
/// that leads to it.
struct Held {
    needs: Vec<String>,
    via: Vec<usize>,
}

/// The roots a need's wait reaches: a waiting node that is not a need, or a
/// need that waits on nothing itself. The `via` chain leaves the root out.
fn roots(infos: &[Option<Info>], start: usize) -> Vec<(usize, Vec<usize>)> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut stack = vec![(start, Vec::new())];
    while let Some((at, via)) = stack.pop() {
        let Some(info) = infos.get(at).and_then(Option::as_ref) else {
            continue;
        };
        for &wait in &info.waiting {
            if !seen.insert(wait) {
                continue;
            }
            match infos.get(wait).and_then(Option::as_ref) {
                Some(next) if next.waiting.is_empty() => found.push((wait, via.clone())),
                Some(_) => {
                    let mut chain = via.clone();
                    chain.push(wait);
                    stack.push((wait, chain));
                }
                None => found.push((wait, via.clone())),
            }
        }
    }
    found
}

fn deciders(snap: &Snapshot) -> Vec<Decider> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for node in snap.nodes() {
        if !question_open(node.data()) {
            continue;
        }
        *counts.entry(decider(node).unwrap_or_default()).or_default() += 1;
    }
    let mut deciders: Vec<Decider> = counts
        .into_iter()
        .map(|(name, count)| Decider { name, count })
        .collect();
    deciders.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.name.cmp(&right.name))
    });
    deciders
}

fn overdue(snap: &Snapshot) -> Vec<Overdue> {
    let today = chrono::Local::now().date_naive();
    let mut overdue: Vec<Overdue> = snap
        .nodes()
        .iter()
        .filter_map(|node| {
            let (due, status) = criterion_wait(node, today)?;
            (status == "due").then(|| Overdue {
                id: node.id().to_string(),
                scope: node.scope().to_string(),
                title: node.title().to_string(),
                due: due.to_string(),
                responsible: node.free("responsible").cloned(),
                then: node.free("then").cloned(),
            })
        })
        .collect();
    overdue.sort_by(|left, right| left.due.cmp(&right.due).then(left.id.cmp(&right.id)));
    overdue
}

/// The open questions nobody waits on and the unmet criteria nobody aims at.
fn unheld(snap: &Snapshot) -> Vec<Unheld> {
    let mut unheld: Vec<(u8, Unheld, u64)> = Vec::new();
    for (at, node) in snap.nodes().iter().enumerate() {
        let kind = node.kind();
        let unheld_node = match kind {
            NodeKind::Question => question_open(node.data()) && unwaited(node, snap.nodes()),
            NodeKind::Criterion => {
                criterion_state(node, snap.nodes()) == CriterionState::Unsatisfied
                    && !aimed_at(snap, at)
            }
            _ => false,
        };
        if !unheld_node {
            continue;
        }
        let state = snap.state(at).unwrap_or_default();
        let order = u8::from(kind == NodeKind::Criterion);
        let created = instant(node.created()).unwrap_or(0);
        unheld.push((
            order,
            Unheld {
                id: node.id().to_string(),
                kind,
                scope: node.scope().to_string(),
                state,
                title: node.title().to_string(),
            },
            created,
        ));
    }
    unheld.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| right.2.cmp(&left.2))
            .then_with(|| left.1.id.cmp(&right.1.id))
    });
    unheld.into_iter().map(|(_, line, _)| line).collect()
}

/// Whether an open need or an in-progress requirement targets this criterion.
fn aimed_at(snap: &Snapshot, at: usize) -> bool {
    snap.arrivals(at).iter().any(|arrival| {
        if arrival.relation != Relation::Targets {
            return false;
        }
        let from = snap.node(arrival.from);
        match from.data() {
            NodeData::Need(_) => need_state(from, snap.nodes()) == NeedState::Open,
            NodeData::Requirement(data) => matches!(
                data.state,
                RequirementState::Filed | RequirementState::Approved
            ),
            _ => false,
        }
    })
}

fn weeks(snap: &Snapshot, needs: &[&Info]) -> Vec<Week> {
    snap.weeks()
        .into_iter()
        .map(|from| {
            let to = from + chrono::Days::new(7);
            let opened = needs
                .iter()
                .filter(|info| {
                    instant(snap.node(info.at).created())
                        .is_some_and(|at| day_of(at) >= from && day_of(at) < to)
                })
                .count();
            let done = needs
                .iter()
                .filter(|info| {
                    info.done_at
                        .is_some_and(|at| day_of(at) >= from && day_of(at) < to)
                })
                .count();
            Week {
                from: from.format("%Y-%m-%d").to_string(),
                opened,
                done,
            }
        })
        .collect()
}

/// Every scope's needs and writes, the busiest first.
fn scopes(snap: &Snapshot, needs: &[&Info]) -> Vec<ScopeLine> {
    let (start, pulse) = snap.pulse(PULSE_DAYS);
    let mut names: Vec<String> = snap
        .nodes()
        .iter()
        .map(|node| node.scope().to_string())
        .collect();
    names.sort();
    names.dedup();
    let mut lines: Vec<ScopeLine> = names
        .into_iter()
        .map(|name| {
            let mine: Vec<&Info> = needs
                .iter()
                .copied()
                .filter(|info| snap.node(info.at).scope() == name)
                .collect();
            let open: Vec<&Info> = mine
                .iter()
                .copied()
                .filter(|info| info.state == NeedState::Open)
                .collect();
            let mut days: Vec<(i64, usize)> = pulse
                .iter()
                .filter(|((scope, _), _)| scope == &name)
                .map(|((_, day), count)| (*day, *count))
                .collect();
            days.sort_unstable();
            ScopeLine {
                open: open.len(),
                closed: mine.len() - open.len(),
                blocked: open.iter().filter(|info| !info.waiting.is_empty()).count(),
                pulse: days
                    .into_iter()
                    .map(|(day, writes)| PulseDay {
                        day: (start + chrono::Days::new(day as u64))
                            .format("%Y-%m-%d")
                            .to_string(),
                        writes,
                    })
                    .collect(),
                name,
            }
        })
        .collect();
    lines.sort_by(|left, right| {
        right
            .open
            .cmp(&left.open)
            .then_with(|| left.name.cmp(&right.name))
    });
    lines
}

fn id(snap: &Snapshot, at: usize) -> String {
    snap.node(at).id().to_string()
}
