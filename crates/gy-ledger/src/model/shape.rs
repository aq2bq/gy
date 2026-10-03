//! The shape of the graph (n-e299, d-858d): the depends-on graph (need to
//! need) and the lineage graph (narrows / widens / supersedes / completes,
//! decision to decision) hold no cycle, a self-loop included. Only an edge a
//! transaction adds is judged, so a ledger that already holds a cycle stays.
use super::rule::{Before, Change};
use super::{Node, NodeKind, Relation};
use crate::store::{Error, Result};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Refuse a transaction that closes a cycle in either graph. Only an updated
/// need or decision whose edges grew is judged; a created node alone cannot
/// close a cycle, because nothing points at it yet.
pub fn admit(before: &dyn Before, changes: &[Change]) -> Result<()> {
    let added = added_edges(before, changes);
    if added.is_empty() {
        return Ok(());
    }
    let graph = overlay(before, changes);
    for (from, relation, to) in added {
        if let Some(path) = reach(&graph, &to, &from) {
            let cycle = cycle(&from, relation, &to, &path);
            return Err(Error::invalid(format!("{cycle} would form a cycle")));
        }
    }
    Ok(())
}

/// The edges the updated needs and decisions add; only the changed node is read.
fn added_edges(before: &dyn Before, changes: &[Change]) -> Vec<(String, &'static str, String)> {
    let mut added = Vec::new();
    for change in changes {
        let Change::Updated(node) = change else {
            continue;
        };
        if !matches!(node.kind(), NodeKind::Need | NodeKind::Decision) {
            continue;
        }
        let after = watched_edges(node);
        if after.is_empty() {
            continue;
        }
        let previous = before
            .get(node.id())
            .map(|node| watched_edges(&node))
            .unwrap_or_default();
        for (relation, target) in after.difference(&previous) {
            added.push((node.id().to_string(), *relation, target.clone()));
        }
    }
    added
}

/// One graph: the previous state with the transaction's nodes; read only when an edge was added.
fn overlay(before: &dyn Before, changes: &[Change]) -> BTreeMap<String, Node> {
    let mut graph: BTreeMap<String, Node> = BTreeMap::new();
    for node in before
        .of_kind(NodeKind::Need)
        .into_iter()
        .chain(before.of_kind(NodeKind::Decision))
    {
        graph.insert(node.id().to_string(), node);
    }
    for change in changes {
        match change {
            Change::Created(node) | Change::Updated(node)
                if matches!(node.kind(), NodeKind::Need | NodeKind::Decision) =>
            {
                graph.insert(node.id().to_string(), node.clone());
            }
            Change::Deleted(id) => {
                graph.remove(&id.to_string());
            }
            _ => {}
        }
    }
    graph
}

/// The forward depends-on and lineage edges: the from side stores the edge.
fn watched_edges(node: &Node) -> BTreeSet<(&'static str, String)> {
    node.links()
        .iter()
        .filter(|edge| !edge.reversed && watched(edge.label))
        .map(|edge| (edge.label.name(), edge.to.to_string()))
        .collect()
}

fn watched(relation: Relation) -> bool {
    matches!(
        relation,
        Relation::DependsOn
            | Relation::Narrows
            | Relation::Widens
            | Relation::Supersedes
            | Relation::Completes
    )
}

/// The edges from `from` back to `to`, or none; `from == to` is the self-loop.
fn reach(
    graph: &BTreeMap<String, Node>,
    from: &str,
    to: &str,
) -> Option<Vec<(&'static str, String)>> {
    if from == to {
        return Some(Vec::new());
    }
    let mut came: BTreeMap<String, (String, &'static str)> = BTreeMap::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    seen.insert(from.to_string());
    queue.push_back(from.to_string());
    while let Some(at) = queue.pop_front() {
        let Some(node) = graph.get(&at) else { continue };
        for edge in node.links() {
            if edge.reversed || !watched(edge.label) {
                continue;
            }
            let next = edge.to.to_string();
            if !seen.insert(next.clone()) {
                continue;
            }
            came.insert(next.clone(), (at.clone(), edge.label.name()));
            if next == to {
                let mut chain = Vec::new();
                let mut at = to.to_string();
                while at != from {
                    let (previous, relation) = &came[&at];
                    chain.push((*relation, at.clone()));
                    at = previous.clone();
                }
                chain.reverse();
                return Some(chain);
            }
            queue.push_back(next);
        }
    }
    None
}

/// The cycle as an ID sequence: the added edge, then the edges leading back.
fn cycle(from: &str, relation: &'static str, to: &str, path: &[(&'static str, String)]) -> String {
    let mut tokens = vec![from.to_string(), relation.to_string(), to.to_string()];
    for (relation, node) in path {
        tokens.push(relation.to_string());
        tokens.push(node.clone());
    }
    tokens.join(" ")
}
