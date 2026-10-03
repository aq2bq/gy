//! The E1–E4 invariant checks the property tests assert (n-99c6): against a
//! pair of snapshots for an accepted step. The test that reads them includes
//! this file with `#[path]`, so a property test that does not judge E1–E4 does
//! not carry it (dead code would otherwise fail the build). `ALLOWED` lives in
//! `prop` because the generator reads it too.
use crate::prop::{ALLOWED, State};
use gy_ledger::{Edge, Node, NodeId, NodeKind, Relation};
use std::collections::BTreeMap;

/// The (from kind, relation, to kind) pairs of `model/links.rs:ALLOWED`.
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
