//! The write-time rule the gate runs (n-557f, n-f921). I1: a criterion may be
//! recorded satisfied only while an approved or done requirement targets it.
//! The coverage judgement is one function, shared with the advice (n-f921).
use super::{Node, NodeData, NodeId, NodeKind, Relation, RequirementState};
use crate::store::{Error, Result};

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
pub trait Before {
    fn get(&self, id: &NodeId) -> Option<Node>;
    fn of_kind(&self, kind: NodeKind) -> Vec<Node>;
}

/// The rule: a criterion recorded satisfied needs an approved or done
/// requirement that targets it (I1, n-f921). Every other change is admitted.
pub fn admit(before: &dyn Before, changes: &[Change]) -> Result<()> {
    if !touches_a_criterion(changes) {
        return Ok(());
    }
    for change in changes {
        if let Change::Created(node) | Change::Updated(node) = change {
            if node.kind() == NodeKind::Criterion {
                check_satisfy(before, node)?;
            }
        }
    }
    Ok(())
}

/// Whether the transaction carries a criterion the rule reads. A requirement
/// change alone reads nothing, so a commit does not decode the whole ledger.
fn touches_a_criterion(changes: &[Change]) -> bool {
    changes.iter().any(|change| match change {
        Change::Created(node) | Change::Updated(node) => node.kind() == NodeKind::Criterion,
        Change::Deleted(_) | Change::ScopeRenamed { .. } => false,
    })
}

/// I1: a criterion whose satisfied goes false to true needs coverage. The only
/// nodes read besides the changed one are the requirements (n-f921).
fn check_satisfy(before: &dyn Before, after: &Node) -> Result<()> {
    if !satisfied(after) || satisfied_of(before.get(after.id()).as_ref()) {
        return Ok(());
    }
    let requirements = before.of_kind(NodeKind::Requirement);
    if covered(&requirements, after.id()) {
        return Ok(());
    }
    Err(Error::invalid(refusal(&requirements, after)))
}

/// The message a refused satisfy reads: what is missing and the one next step.
fn refusal(requirements: &[Node], criterion: &Node) -> String {
    let step = match filed_target(requirements, criterion.id()) {
        Some(request) => format!(
            "req approve {} --design … --heard-by … --evidence …",
            request.id()
        ),
        None => format!(
            "req add \"<title>\" --need <N> --targets {}",
            criterion.id()
        ),
    };
    format!(
        "{} is not covered by an approved requirement (targets); {step}",
        criterion.id()
    )
}

/// The one coverage judgement (n-f921): whether an approved or done
/// requirement targets `criterion`. The rule and the advice share it.
pub fn covered(nodes: &[Node], criterion: &NodeId) -> bool {
    covering(nodes, criterion).is_some()
}

/// The approved or done requirement that targets `criterion`, if any.
pub fn covering<'a>(nodes: &'a [Node], criterion: &NodeId) -> Option<&'a Node> {
    nodes
        .iter()
        .find(|node| is_approved(node) && targets(node, criterion))
}

/// The filed requirement that targets `criterion`, if any: the one an approve
/// would turn into coverage (n-f921).
pub fn filed_target<'a>(nodes: &'a [Node], criterion: &NodeId) -> Option<&'a Node> {
    nodes
        .iter()
        .find(|node| node.state() == Some(RequirementState::Filed) && targets(node, criterion))
}

fn is_approved(node: &Node) -> bool {
    matches!(
        node.state(),
        Some(RequirementState::Approved | RequirementState::Done)
    )
}

fn targets(node: &Node, criterion: &NodeId) -> bool {
    node.links()
        .iter()
        .any(|edge| edge.label == Relation::Targets && &edge.to == criterion)
}

fn satisfied(node: &Node) -> bool {
    matches!(node.data(), NodeData::Criterion(data) if data.satisfied)
}

fn satisfied_of(node: Option<&Node>) -> bool {
    node.is_some_and(satisfied)
}
