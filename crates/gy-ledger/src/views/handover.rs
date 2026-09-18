//! handover: what a session needs to resume in one command (AC-43). In-progress
//! requirements with their ref and who waits, the counts that route the next
//! step, integrity errors, and warnings as counts only (proposal-v3 14).
use super::derive::{edges, find, ready, reference, requirement_in_progress, writer};
use super::sync_row::{SyncRow, sync_row};
use crate::model::{Node, NodeData, NodeKind, Relation};
use crate::ops::advice;
use crate::ops::repository::{Repository, Result, Store};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// An in-progress requirement: id, ref, state, title, and who waits on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProgressRow {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub state: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsible: Option<String>,
    /// The writer of the requirement's last write, as a reader sees them
    /// (n-d36d).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,
}
impl fmt::Display for ProgressRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match &self.reference {
            Some(reference) => format!("{} ({reference})", self.id),
            None => self.id.clone(),
        };
        writeln!(f, "{id} {} {}", self.state, self.title)?;
        if let Some(next_evidence) = &self.next_evidence {
            writeln!(f, "  next_evidence: {next_evidence}")?;
        }
        if let Some(responsible) = &self.responsible {
            writeln!(f, "  responsible: {responsible}")?;
        }
        if let Some(who) = &self.who {
            writeln!(f, "  who: {who}")?;
        }
        Ok(())
    }
}

/// A warning kind and how many there are. The list itself is not shown (AC-43).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Warning {
    pub label: String,
    pub count: usize,
}
impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.label, self.count)
    }
}

/// The session handover: errors first, then in-progress, then the counts that
/// route the next step, then warning counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Handover {
    pub errors: Vec<String>,
    /// The synced copy's state, when one is due (n-ecbf).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<SyncRow>,
    pub in_progress: Vec<ProgressRow>,
    pub open_questions: usize,
    pub ready_needs: usize,
    pub warnings: Vec<Warning>,
}
impl fmt::Display for Handover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.errors.is_empty() {
            writeln!(f, "errors:")?;
            for error in &self.errors {
                writeln!(f, "  {error}")?;
            }
        }
        if let Some(sync) = &self.sync {
            writeln!(f, "{sync}")?;
        }
        for row in &self.in_progress {
            write!(f, "{row}")?;
        }
        writeln!(f, "in progress: {}", self.in_progress.len())?;
        writeln!(f, "open questions: {}", self.open_questions)?;
        writeln!(f, "ready needs: {}", self.ready_needs)?;
        for warning in &self.warnings {
            writeln!(f, "{warning}")?;
        }
        Ok(())
    }
}

pub fn handover<S: Store>(repo: &Repository<S>, scope: Option<&str>) -> Result<Handover> {
    let all = repo.all()?;
    let in_progress: Vec<&Node> = all
        .iter()
        .filter(|node| in_scope(node, scope) && requirement_in_progress(node))
        .collect();
    let open_questions = all
        .iter()
        .filter(|node| in_scope(node, scope) && open_question(node))
        .count();
    let ready_needs = all
        .iter()
        .filter(|node| in_scope(node, scope) && node.kind() == NodeKind::Need && ready(node, &all))
        .count();
    let who = last_writers(repo);
    Ok(Handover {
        errors: errors_with_rejected(&all, repo),
        sync: sync_row(repo),
        in_progress: in_progress
            .iter()
            .map(|node| progress(node, who.get(&node.id().to_string()).cloned()))
            .collect(),
        open_questions,
        ready_needs,
        warnings: warnings(&all, scope, &in_progress),
    })
}

/// Integrity errors, then the refused writes this copy still carries
/// (n-ecbf 2B2). A local ledger has none.
fn errors_with_rejected<S: Store>(all: &[Node], repo: &Repository<S>) -> Vec<String> {
    let mut errors = integrity(all);
    errors.extend(repo.store().rejected());
    errors
}

/// Who wrote each node last, as a reader sees them (n-d36d). A scope rename
/// names no node, so it is left out.
fn last_writers<S: Store>(repo: &Repository<S>) -> BTreeMap<String, String> {
    let mut who = BTreeMap::new();
    for entry in repo.store().history() {
        if entry.node.is_empty() {
            continue;
        }
        let actor = entry.actor.name();
        who.insert(entry.node.clone(), writer(entry.by.as_deref(), actor));
    }
    who
}

fn progress(node: &Node, who: Option<String>) -> ProgressRow {
    ProgressRow {
        id: node.id().to_string(),
        reference: reference(node),
        state: node
            .state()
            .map(|state| state.name().to_string())
            .unwrap_or_default(),
        title: node.title().to_string(),
        next_evidence: node.free("next_evidence").cloned(),
        responsible: node.free("responsible").cloned(),
        who,
    }
}

/// Edges that point at nothing, and filed-as edges whose target is not a
/// requirement. Always-valid writes make these rare (D-75).
fn integrity(all: &[Node]) -> Vec<String> {
    let known: BTreeSet<String> = all.iter().map(|node| node.id().to_string()).collect();
    let mut errors = Vec::new();
    for node in all {
        for edge in node.links() {
            if !known.contains(&edge.to.to_string()) {
                errors.push(format!("{} {} missing {}", node.id(), edge.name(), edge.to));
            } else if edge.label == Relation::FiledAs
                && !find(all, &edge.to).is_some_and(|node| node.kind() == NodeKind::Requirement)
            {
                errors.push(format!(
                    "{} is filed as {}, which is not a requirement",
                    node.id(),
                    edge.to
                ));
            }
        }
    }
    errors
}

fn warnings(all: &[Node], scope: Option<&str>, in_progress: &[&Node]) -> Vec<Warning> {
    let mut warnings = Vec::new();
    push(
        &mut warnings,
        "in-progress requirements relying on a superseded decision",
        relies_on(in_progress, &node_ids(all, Relation::Supersedes)),
    );
    push(
        &mut warnings,
        "in-progress requirements relying on an unrecorded decision scope",
        relies_on(in_progress, &unrecorded_ids(all)),
    );
    push(
        &mut warnings,
        "criteria with an empty body",
        empty_bodies(all, scope),
    );
    push(
        &mut warnings,
        "open questions nobody waits on",
        nobody_waits(all, scope),
    );
    push(
        &mut warnings,
        "criteria unmet with every need closed",
        orphaned_criteria(all, scope),
    );
    warnings
}

/// The ids targeted by one relation anywhere in the graph.
fn node_ids(all: &[Node], relation: Relation) -> BTreeSet<String> {
    all.iter()
        .flat_map(|node| edges(node, relation))
        .map(|id| id.to_string())
        .collect()
}

fn unrecorded_ids(all: &[Node]) -> BTreeSet<String> {
    all.iter()
        .filter(|node| unrecorded_scope(node))
        .map(|node| node.id().to_string())
        .collect()
}

fn relies_on(in_progress: &[&Node], targets: &BTreeSet<String>) -> usize {
    in_progress
        .iter()
        .filter(|node| {
            edges(node, Relation::ReliesOn)
                .iter()
                .any(|id| targets.contains(&id.to_string()))
        })
        .count()
}

fn empty_bodies(all: &[Node], scope: Option<&str>) -> usize {
    all.iter()
        .filter(|node| {
            in_scope(node, scope)
                && node.kind() == NodeKind::Criterion
                && node.body().trim().is_empty()
        })
        .count()
}

/// Open questions in scope that no node waits on or raised (n-fa11, d-09b6).
fn nobody_waits(all: &[Node], scope: Option<&str>) -> usize {
    all.iter()
        .filter(|node| in_scope(node, scope) && advice::unwaited(node, all))
        .count()
}

/// Criteria in scope whose every bearing need is closed (n-f7ef, d-f7b6).
fn orphaned_criteria(all: &[Node], scope: Option<&str>) -> usize {
    all.iter()
        .filter(|node| in_scope(node, scope) && advice::unmet_orphaned(node, all))
        .count()
}

fn push(warnings: &mut Vec<Warning>, label: &str, count: usize) {
    if count > 0 {
        warnings.push(Warning {
            label: label.to_string(),
            count,
        });
    }
}

fn in_scope(node: &Node, scope: Option<&str>) -> bool {
    scope.is_none_or(|scope| node.scope() == scope)
}

fn open_question(node: &Node) -> bool {
    matches!(node.data(), NodeData::Question(data) if data.closure.is_none())
}

fn unrecorded_scope(node: &Node) -> bool {
    matches!(node.data(), NodeData::Decision(data) if data.scope.is_unrecorded())
}
