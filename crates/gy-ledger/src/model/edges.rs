//! The gate's edge rules (d-fb49): a transaction may add neither a second copy
//! of an edge nor a closes edge on a question no decision closed. Every write,
//! a create operation or a rebase line, passes here. A state the ledger already
//! held stays.
use super::rule::{Before, Change};
use super::{Closure, Node, NodeData, NodeKind, Relation};
use crate::store::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Refuse a transaction that adds a duplicate edge or a non-decision closes.
/// The previous state is read only for a node that adds one of those.
pub fn admit(before: &dyn Before, changes: &[Change]) -> Result<()> {
    for change in changes {
        let (Change::Created(node) | Change::Updated(node)) = change else {
            continue;
        };
        check_duplicate(before, node)?;
        check_closes(before, node)?;
    }
    Ok(())
}

/// Refuse an edge the write adds that copies one already there; a duplicate the
/// ledger already held (its count did not grow) stays.
fn check_duplicate(before: &dyn Before, node: &Node) -> Result<()> {
    let after = forward_edges(node);
    if !after.values().any(|count| *count > 1) {
        return Ok(());
    }
    let held = before
        .get(node.id())
        .map(|previous| forward_edges(&previous))
        .unwrap_or_default();
    for (key, count) in after {
        if count > 1 && count > held.get(&key).copied().unwrap_or(0) {
            return Err(Error::invalid(format!(
                "{} {} {} is already registered",
                node.id(),
                key.0,
                key.1
            )));
        }
    }
    Ok(())
}

fn forward_edges(node: &Node) -> BTreeMap<(&'static str, String), usize> {
    let mut counts = BTreeMap::new();
    for edge in node.links().iter().filter(|edge| !edge.reversed) {
        *counts
            .entry((edge.label.name(), edge.to.to_string()))
            .or_insert(0) += 1;
    }
    counts
}

/// Refuse a closes edge the write adds when no decision closed the question; a
/// closes edge the ledger already held stays.
fn check_closes(before: &dyn Before, node: &Node) -> Result<()> {
    if node.kind() != NodeKind::Question || closure(node) == Some(Closure::Decision) {
        return Ok(());
    }
    let after = closes(node);
    if after.is_empty() {
        return Ok(());
    }
    let held = before
        .get(node.id())
        .map(|previous| closes(&previous))
        .unwrap_or_default();
    if after.difference(&held).next().is_none() {
        return Ok(());
    }
    Err(Error::invalid(
        "closes is made only when the question is closed by decision; name the decision in --evidence",
    ))
}

fn closes(node: &Node) -> BTreeSet<String> {
    node.links()
        .iter()
        .filter(|edge| !edge.reversed && edge.label == Relation::Closes)
        .map(|edge| edge.to.to_string())
        .collect()
}

fn closure(node: &Node) -> Option<Closure> {
    match node.data() {
        NodeData::Question(data) => data.closure,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Before, Change, Link, Node, NodeId, NodeKind, Relation};
    use super::admit;
    use std::cell::Cell;

    /// A view of the previous state that counts every read of it.
    struct Counted<'a> {
        reads: &'a Cell<usize>,
    }
    impl Before for Counted<'_> {
        fn get(&self, _id: &NodeId) -> Option<Node> {
            self.reads.set(self.reads.get() + 1);
            None
        }
        fn of_kind(&self, _kind: NodeKind) -> Vec<Node> {
            Vec::new()
        }
    }

    fn need(hash: &str) -> Node {
        let id = NodeId::from_hash(NodeKind::Need, hash).unwrap();
        Node::need(id, "a", "2026-09-15T00:00:00Z", "a need").unwrap()
    }

    /// The rule looks only at what the write changes: a write that adds no
    /// duplicate edge and closes no question reads no previous state.
    #[test]
    fn a_write_that_adds_no_duplicate_reads_no_previous_state() {
        let reads = Cell::new(0);
        admit(&Counted { reads: &reads }, &[Change::Created(need("0001"))]).unwrap();
        let mut node = need("0002");
        let to = NodeId::from_hash(NodeKind::Need, "0001").unwrap();
        node.link(Link::new(node.id().clone(), Relation::DependsOn, to).unwrap());
        admit(&Counted { reads: &reads }, &[Change::Created(node)]).unwrap();
        assert_eq!(reads.get(), 0);
    }
}
