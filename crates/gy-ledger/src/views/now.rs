//! now: what the master is waiting on, what is in progress, and what is still
//! open (d-995c, n-688a). The one place this judgement lives, so the CLI, MCP,
//! and serve give the same answer.
use super::derive::{need_state, question_open, reference};
use super::list::LogRow;
use super::{NeedState, open_or_closed};
use crate::model::{Node, NodeData, NodeKind, RequirementState};
use crate::ops::repository::{Repository, Result, Store};
use serde::Serialize;

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
}

/// One thing the master waits on: an open question whose decider names the
/// master, or a filed requirement waiting for the design approval (d-995c).
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
    pub in_progress: Vec<NodeRow>,
    pub open_questions: Vec<NodeRow>,
    pub unmet: Vec<NodeRow>,
    pub recent: Vec<LogRow>,
}

pub fn now<S: Store>(repo: &Repository<S>, scope: Option<&str>) -> Result<Now> {
    let all = repo.all()?;
    let visible: Vec<&Node> = all.iter().filter(|node| in_scope(node, scope)).collect();
    let mut in_progress: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| node.kind() == NodeKind::Need && need_state(node, &all) == NeedState::Open)
        .collect();
    in_progress.sort_by(newest_first);
    let mut open_questions: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| question_open(node) && !names_master(node))
        .collect();
    open_questions.sort_by(oldest_first);
    let mut unmet: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| matches!(node.data(), NodeData::Criterion(data) if !data.satisfied))
        .collect();
    unmet.sort_by(newest_first);
    let last = repo.store().history().last();
    Ok(Now {
        seq: last.map_or(0, |entry| entry.seq),
        at: last.map_or(0, |entry| entry.at),
        scope: scope.map(ToString::to_string),
        waiting: waiting(&visible, &all),
        in_progress: rows(&in_progress, &all),
        open_questions: rows(&open_questions, &all),
        unmet: rows(&unmet, &all),
        recent: recent(repo, scope, &all),
    })
}

/// The master's queue: the questions that name the master, then the filed
/// requirements, each in created order (d-995c).
fn waiting(visible: &[&Node], all: &[Node]) -> Vec<Waiting> {
    let mut questions: Vec<&Node> = visible
        .iter()
        .copied()
        .filter(|node| names_master(node))
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

/// Whether an open question names the master as its decider.
fn names_master(node: &Node) -> bool {
    matches!(node.data(), NodeData::Question(data)
        if question_open(node) && data.decider.as_deref().is_some_and(names_the_master))
}

/// The two habits of both ledgers: `master` in any case, and `マスター`.
fn names_the_master(decider: &str) -> bool {
    decider.to_lowercase().contains("master") || decider.contains("マスター")
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

/// The last writes, newest first, fourteen of them. With a scope, only writes
/// to that scope's nodes (a scope rename names no node, so it is left out).
fn recent<S: Store>(repo: &Repository<S>, scope: Option<&str>, all: &[Node]) -> Vec<LogRow> {
    repo.store()
        .history()
        .iter()
        .rev()
        .filter(|entry| scope.is_none_or(|scope| scope_of(all, &entry.node) == Some(scope)))
        .take(RECENT)
        .map(|entry| LogRow {
            seq: entry.seq,
            at: entry.at,
            actor: entry.actor.name().to_string(),
            node: entry.node.clone(),
            what: entry.what.clone(),
            why: entry.why.clone(),
            source: entry.source.clone(),
        })
        .collect()
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
