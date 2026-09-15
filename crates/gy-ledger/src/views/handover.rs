//! handover: what a session needs to resume in one command (AC-43). In-progress
//! requirements with their ref and who waits, the counts that route the next
//! step, integrity errors, and warnings as counts only (proposal-v3 14).
use super::derive::{edges, find, ready, reference, requirement_in_progress};
use crate::model::{Node, NodeData, NodeKind, Relation};
use crate::ops::repository::{Repository, Result, Store};
use serde::Serialize;
use std::collections::BTreeSet;
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
    Ok(Handover {
        errors: integrity(&all),
        in_progress: in_progress.iter().map(|node| progress(node)).collect(),
        open_questions,
        ready_needs,
        warnings: warnings(&all, scope, &in_progress),
    })
}

fn progress(node: &Node) -> ProgressRow {
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
