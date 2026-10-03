//! The gaps an open need reports: its body, a filed-as requirement, and the
//! targeted criteria the requirements that closed left behind (n-f60a, d-85c6).
use super::{body_gap, find, linked, unsatisfied};
use crate::model::{Node, NodeId, Relation, RequirementState};

pub(super) fn need_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    if linked(node, Relation::FiledAs).is_empty() {
        out.push("a filed-as requirement".to_string());
    } else {
        out.extend(
            unmet_criteria(node, all)
                .into_iter()
                .map(|criterion| format!("unmet criterion {}", criterion.id())),
        );
    }
    out
}

/// The criteria a need targets that are unsatisfied and that no requirement
/// still being built (`filed` or `approved`) targets: the gaps the requirements
/// that closed left behind. Only a need with a filed-as requirement reports
/// them, so a new need keeps saying it needs one.
pub(super) fn unmet_criteria<'a>(need: &Node, all: &'a [Node]) -> Vec<&'a Node> {
    linked(need, Relation::Targets)
        .into_iter()
        .filter_map(|id| find(all, &id))
        .filter(|criterion| unsatisfied(criterion))
        .filter(|criterion| !in_progress_targets(all, criterion.id()))
        .collect()
}

/// Whether a requirement still being built targets `criterion`.
fn in_progress_targets(all: &[Node], criterion: &NodeId) -> bool {
    all.iter().any(|node| {
        matches!(
            node.state(),
            Some(RequirementState::Filed | RequirementState::Approved)
        ) && linked(node, Relation::Targets).contains(criterion)
    })
}
