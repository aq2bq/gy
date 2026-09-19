//! now: what a person is waiting on, what is in progress, and what is still
//! open (d-995c, d-b02d, n-688a). The CLI, MCP, and serve give the same answer.
use super::derive::{need_state, question_open, reference, writer, writers};
use super::handover::{Handover, ProgressRow, handover};
use super::list::LogRow;
use super::next::ready_rows;
use super::sync_row::SyncRow;
use super::{NeedState, open_or_closed};
use crate::model::{Node, NodeData, NodeKind, RequirementState};
use crate::ops::advice;
use crate::ops::repository::{Repository, Result, Store};
use serde::Serialize;
use std::collections::BTreeSet;

/// How many recent writes the view carries (d-995c).
const RECENT: usize = 14;

/// One node as now lists it: id, first alias, kind, title, scope, and the
/// model's own name for its state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeRow {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub kind: NodeKind,
    pub title: String,
    pub scope: String,
    pub status: String,
    pub created: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub unwaited: bool,
}

/// One thing a person waits on: an open question whose decider never wrote the
/// ledger, or a filed requirement waiting for the design approval (d-995c, d-b02d).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "waiting", rename_all = "kebab-case")]
pub enum Waiting {
    Question {
        row: NodeRow,
        decider: String,
        options: Vec<String>,
    },
    Requirement {
        row: NodeRow,
        #[serde(skip_serializing_if = "Option::is_none")]
        reference: Option<String>,
    },
}

/// One need ready to work, exactly as `next` orders it, with its criteria
/// counts (d-3e8f).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ready {
    pub row: NodeRow,
    pub satisfied: usize,
    pub targets: usize,
}

/// How a session resumes: the in-progress requirements, the counts that route
/// the next step, and the last write (d-3e8f).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resume {
    pub in_progress: Vec<ProgressRow>,
    pub open_questions: usize,
    pub warnings: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<SyncRow>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<LogRow>,
}

/// The now view: where the log stands, who waits, what is in progress, what is
/// still open, and the writes that just happened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Now {
    pub seq: u64,
    /// The last write, in epoch seconds.
    pub at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub waiting: Vec<Waiting>,
    pub ready: Vec<Ready>,
    pub resume: Resume,
    pub in_progress: Vec<NodeRow>,
    pub open_questions: Vec<NodeRow>,
    pub unmet: Vec<NodeRow>,
    pub recent: Vec<LogRow>,
}

pub fn now<S: Store>(repo: &Repository<S>, scope: Option<&str>) -> Result<Now> {
    let all = repo.all()?;
    let visible: Vec<&Node> = all.iter().filter(|node| in_scope(node, scope)).collect();
    let writers = writers(repo);
    let mut in_progress: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| node.kind() == NodeKind::Need && need_state(node, &all) == NeedState::Open)
        .collect();
    in_progress.sort_by(newest_first);
    let mut open_questions: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| question_open(node) && !waits_on_a_person(node, &writers))
        .collect();
    open_questions.sort_by(oldest_first);
    let mut unmet: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| matches!(node.data(), NodeData::Criterion(data) if !data.satisfied))
        .collect();
    unmet.sort_by(newest_first);
    let last = repo.store().history().last();
    let session = handover(repo, scope)?;
    Ok(Now {
        seq: last.map_or(0, |entry| entry.seq),
        at: last.map_or(0, |entry| entry.at),
        scope: scope.map(ToString::to_string),
        waiting: waiting(&visible, &all, &writers),
        ready: ready(&all, scope),
        resume: resume(repo, scope, &all, session),
        in_progress: rows(&in_progress, &all),
        open_questions: rows(&open_questions, &all),
        unmet: rows(&unmet, &all),
        recent: recent(repo, scope, &all),
    })
}

/// The resume eye, from the handover view plus the last write (d-3e8f).
fn resume<S: Store>(
    repo: &Repository<S>,
    scope: Option<&str>,
    all: &[Node],
    session: Handover,
) -> Resume {
    Resume {
        sync: session.sync.clone(),
        errors: session.errors.clone(),
        in_progress: session.in_progress,
        open_questions: session.open_questions,
        warnings: session.warnings.iter().map(|warning| warning.count).sum(),
        last: last_row(repo, scope, all),
    }
}

fn ready(all: &[Node], scope: Option<&str>) -> Vec<Ready> {
    ready_rows(all, scope)
        .into_iter()
        .filter_map(|next| {
            let node = all.iter().find(|node| node.id().to_string() == next.id)?;
            Some(Ready {
                row: row(node, all),
                satisfied: next.satisfied,
                targets: next.targets,
            })
        })
        .collect()
}

/// A person's queue: the open questions whose decider has never written, then
/// the filed requirements, each in created order (d-995c, d-b02d).
fn waiting(visible: &[&Node], all: &[Node], writers: &BTreeSet<String>) -> Vec<Waiting> {
    let mut questions: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| waits_on_a_person(node, writers))
        .collect();
    questions.sort_by(oldest_first);
    let mut requirements: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| is_filed(node))
        .collect();
    requirements.sort_by(oldest_first);
    questions
        .iter()
        .map(|node| question_waiting(node, all))
        .chain(
            requirements
                .iter()
                .map(|node| requirement_waiting(node, all)),
        )
        .collect()
}

/// Whether an open question waits on a person: a decider that is set, and that
/// never wrote the ledger (d-b02d).
fn waits_on_a_person(node: &Node, writers: &BTreeSet<String>) -> bool {
    matches!(node.data(), NodeData::Question(data)
        if question_open(node)
            && data.decider.as_deref().map(str::trim)
                .is_some_and(|decider| !decider.is_empty() && !writers.contains(decider)))
}

/// A requirement that is filed waits for the design approval (D-67).
fn is_filed(node: &Node) -> bool {
    matches!(node.data(), NodeData::Requirement(data) if data.state == RequirementState::Filed)
}

fn question_waiting(node: &Node, all: &[Node]) -> Waiting {
    let NodeData::Question(data) = node.data() else {
        unreachable!("only a question waits as a question")
    };
    Waiting::Question {
        row: row(node, all),
        decider: data.decider.clone().unwrap_or_default(),
        options: data.options.clone(),
    }
}

fn requirement_waiting(node: &Node, all: &[Node]) -> Waiting {
    Waiting::Requirement {
        row: row(node, all),
        reference: reference(node),
    }
}

fn rows(nodes: &[&Node], all: &[Node]) -> Vec<NodeRow> {
    nodes.iter().map(|node| row(node, all)).collect()
}

fn row(node: &Node, all: &[Node]) -> NodeRow {
    NodeRow {
        id: node.id().to_string(),
        alias: node.aliases().first().map(|alias| alias.0.clone()),
        kind: node.kind(),
        title: node.title().to_string(),
        scope: node.scope().to_string(),
        status: state_name(node, all),
        created: node.created().to_string(),
        unwaited: advice::unwaited(node, all),
    }
}

/// The name the model gives the node's state; the words list prints too.
fn state_name(node: &Node, all: &[Node]) -> String {
    match node.data() {
        NodeData::Need(_) => need_state(node, all).name().to_string(),
        NodeData::Question(data) => open_or_closed(data.closure.is_some()).to_string(),
        NodeData::Requirement(data) => data.state.name().to_string(),
        NodeData::Criterion(data) => if data.satisfied {
            "satisfied"
        } else {
            "unsatisfied"
        }
        .to_string(),
        NodeData::Decision(_) => String::new(),
    }
}

/// The last writes, newest first, fourteen, in scope (n-3e8f).
fn recent<S: Store>(repo: &Repository<S>, scope: Option<&str>, all: &[Node]) -> Vec<LogRow> {
    repo.store()
        .history()
        .iter()
        .rev()
        .filter(|entry| scope.is_none_or(|scope| scope_of(all, &entry.node) == Some(scope)))
        .take(RECENT)
        .map(|entry| {
            let actor = entry.actor.name().to_string();
            LogRow {
                seq: entry.seq,
                at: entry.at,
                who: writer(entry.by.as_deref(), &actor),
                actor,
                node: entry.node.clone(),
                what: entry.what.clone(),
                why: entry.why.clone(),
                source: entry.source.clone(),
            }
        })
        .collect()
}

fn last_row<S: Store>(repo: &Repository<S>, scope: Option<&str>, all: &[Node]) -> Option<LogRow> {
    recent(repo, scope, all).into_iter().next()
}

fn scope_of<'a>(all: &'a [Node], id: &str) -> Option<&'a str> {
    all.iter()
        .find(|node| node.id().to_string() == id)
        .map(|node| node.scope())
}

/// Older first, the id breaking ties so the order is total.
fn oldest_first(a: &&Node, b: &&Node) -> std::cmp::Ordering {
    a.created()
        .cmp(b.created())
        .then_with(|| a.id().to_string().cmp(&b.id().to_string()))
}

fn newest_first(a: &&Node, b: &&Node) -> std::cmp::Ordering {
    oldest_first(b, a)
}

fn in_scope(node: &Node, scope: Option<&str>) -> bool {
    scope.is_none_or(|scope| node.scope() == scope)
}
