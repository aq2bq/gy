//! The E1–E4 invariant checks the property tests assert (n-99c6): against a
//! pair of snapshots for an accepted step. E1's table is copied here on
//! purpose, so a change to the implementation table shows up as a failure.
use crate::prop::State;
use gy_ledger::{Edge, Node, NodeData, NodeId, NodeKind, Relation, RequirementState};
use std::collections::BTreeMap;

/// The (from kind, relation, to kind) pairs of `model/links.rs:ALLOWED`.
pub const ALLOWED: &[(Relation, NodeKind, NodeKind)] = &[
    (Relation::Closes, NodeKind::Question, NodeKind::Decision),
    (Relation::Narrows, NodeKind::Decision, NodeKind::Decision),
    (Relation::Widens, NodeKind::Decision, NodeKind::Decision),
    (Relation::Supersedes, NodeKind::Decision, NodeKind::Decision),
    (Relation::Completes, NodeKind::Decision, NodeKind::Decision),
    (Relation::Targets, NodeKind::Need, NodeKind::Criterion),
    (
        Relation::Targets,
        NodeKind::Requirement,
        NodeKind::Criterion,
    ),
    (Relation::SpawnedBy, NodeKind::Need, NodeKind::Decision),
    (Relation::FiledAs, NodeKind::Need, NodeKind::Requirement),
    (Relation::DependsOn, NodeKind::Need, NodeKind::Need),
    (
        Relation::ReliesOn,
        NodeKind::Requirement,
        NodeKind::Decision,
    ),
    (Relation::Raised, NodeKind::Requirement, NodeKind::Question),
    (Relation::WaitsOn, NodeKind::Need, NodeKind::Question),
    (Relation::WaitsOn, NodeKind::Need, NodeKind::Requirement),
];

pub fn allowed(from: NodeKind, relation: Relation, to: NodeKind) -> bool {
    ALLOWED
        .iter()
        .any(|(rel, src, dst)| *rel == relation && *src == from && *dst == to)
}

pub fn find<'a>(nodes: &'a [Node], id: &NodeId) -> Option<&'a Node> {
    nodes.iter().find(|node| node.id() == id)
}

/// The forward edges of a node: the from side is the only side stored.
pub fn fwd(node: &Node) -> impl Iterator<Item = &Edge> {
    node.links().iter().filter(|edge| !edge.reversed)
}

pub fn is_approved(node: &Node) -> bool {
    node.state() == Some(RequirementState::Approved)
}

pub fn is_closed_need(node: &Node) -> bool {
    matches!(node.data(), NodeData::Need(data) if data.closed.is_some())
}

fn key(edge: &Edge) -> (String, &'static str, String) {
    (
        edge.from.to_string(),
        edge.label.name(),
        edge.to.to_string(),
    )
}

fn counts(nodes: &[Node]) -> BTreeMap<(String, &'static str, String), usize> {
    let mut map = BTreeMap::new();
    for node in nodes {
        for edge in fwd(node) {
            *map.entry(key(edge)).or_insert(0) += 1;
        }
    }
    map
}

/// E1: every edge's pair is in the table.
pub fn broken_e1(_before: &State, after: &State) -> bool {
    after
        .nodes
        .iter()
        .any(|node| fwd(node).any(|edge| !allowed(node.kind(), edge.label, edge.to.kind())))
}

/// E2: every stored edge is on its from side, forward.
pub fn broken_e2(_before: &State, after: &State) -> bool {
    after.nodes.iter().any(|node| {
        node.links()
            .iter()
            .any(|edge| edge.reversed || &edge.from != node.id())
    })
}

/// E3: no edge key gained a second copy.
pub fn broken_e3(before: &State, after: &State) -> bool {
    let (was, now) = (counts(&before.nodes), counts(&after.nodes));
    now.iter()
        .any(|(key, count)| *count > was.get(key).copied().unwrap_or(0).max(1))
}

/// E4: every edge's target exists.
pub fn broken_e4(_before: &State, after: &State) -> bool {
    after
        .nodes
        .iter()
        .any(|node| fwd(node).any(|edge| find(&after.nodes, &edge.to).is_none()))
}
