//! The gaps an open need reports: its body, a filed-as requirement, and the
//! targeted criteria the requirements that closed left behind (n-f60a, d-85c6).
use super::body_gap;
use crate::model::{
    Node, NodeData, NodeId, Relation, RequirementState, criterion_unsatisfied, edges, find,
    requirement_in_progress,
};

pub(super) fn need_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    if has_live_filed_as(node, all) {
        out.extend(
            unmet_criteria(node, all)
                .into_iter()
                .map(|criterion| format!("unmet criterion {}", criterion.id())),
        );
    } else {
        out.push("a filed-as requirement".to_string());
    }
    out
}

/// Whether the need has a filed-as requirement a write can still count on: a
/// requirement that is not cancelled. A cancelled requirement leaves the need
/// as if it had none, so it asks for a new one (d-bf90).
fn has_live_filed_as(node: &Node, all: &[Node]) -> bool {
    edges(node, Relation::FiledAs)
        .into_iter()
        .filter_map(|id| find(all, &id))
        .any(|other| {
            matches!(
                other.data(),
                NodeData::Requirement(data) if data.state != RequirementState::Cancelled
            )
        })
}

/// The criteria a need targets that are unsatisfied and that no requirement
/// still being built (`filed` or `approved`) targets: the gaps the requirements
/// that closed left behind. Only a need with a filed-as requirement reports
/// them, so a new need keeps saying it needs one.
pub(super) fn unmet_criteria<'a>(need: &Node, all: &'a [Node]) -> Vec<&'a Node> {
    edges(need, Relation::Targets)
        .into_iter()
        .filter_map(|id| find(all, &id))
        .filter(|criterion| criterion_unsatisfied(criterion.data()))
        .filter(|criterion| !in_progress_targets(all, criterion.id()))
        .collect()
}

/// Whether a requirement still being built targets `criterion`.
fn in_progress_targets(all: &[Node], criterion: &NodeId) -> bool {
    all.iter().any(|node| {
        requirement_in_progress(node.data()) && edges(node, Relation::Targets).contains(criterion)
    })
}
