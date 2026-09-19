//! The write-time rule the gate runs (n-557f). The rule is empty here: it
//! admits every change; the next need fills it in.
use super::{Node, NodeId, NodeKind};
use crate::store::Result;

/// A change, typed: what the store's log carries as JSON, for the rule to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Created(Node),
    Updated(Node),
    Deleted(NodeId),
    ScopeRenamed { from: String, to: String },
}

/// The node state before a change, as the rule reads it (n-557f). The gate
/// builds this over the stored JSON and turns only what the rule asks for into
/// a node, so a commit does not decode the whole ledger.
#[allow(dead_code)] // the empty rule reads nothing; the next need calls these
pub trait Before {
    fn get(&self, id: &NodeId) -> Option<Node>;
    fn of_kind(&self, kind: NodeKind) -> Vec<Node>;
}

/// The rule: n-557f leaves it open, so every change is admitted.
pub fn admit(_before: &dyn Before, _changes: &[Change]) -> Result<()> {
    Ok(())
}
