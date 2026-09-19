//! The write-time rule the gate runs (n-557f, n-f921, n-a3f2). I1: a criterion
//! may be recorded satisfied only while an approved or done requirement targets
//! it. I2: while a requirement stays approved, its title, body, and its targets
//! and relies-on edges are frozen, and so are the title and body of a criterion
//! it targets; `req revise` first. The coverage judgement is one function,
//! shared with the advice (n-f921).
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

/// The rule: I1 for a criterion turning satisfied, and the n-a3f2 freeze for an
/// approved requirement and the criteria it targets. Every other change is
/// admitted.
pub fn admit(before: &dyn Before, changes: &[Change]) -> Result<()> {
    if !reads_state(changes) {
        return Ok(());
    }
    for change in changes {
        match change {
            Change::Updated(node) => match node.kind() {
                NodeKind::Criterion => check_criterion(before, node)?,
                NodeKind::Requirement => check_frozen_requirement(before, node)?,
                _ => {}
            },
            Change::Created(node) if node.kind() == NodeKind::Criterion => {
                check_criterion(before, node)?
            }
            _ => {}
        }
    }
    Ok(())
}

/// Whether the transaction carries a criterion or requirement the rule reads.
/// Any other transaction reads no previous state, so a commit decodes nothing
/// it does not judge (n-f921, n-a3f2).
fn reads_state(changes: &[Change]) -> bool {
    changes.iter().any(|change| match change {
        Change::Created(node) | Change::Updated(node) => {
            matches!(node.kind(), NodeKind::Criterion | NodeKind::Requirement)
        }
        Change::Deleted(_) | Change::ScopeRenamed { .. } => false,
    })
}

/// I1 on a new satisfaction and, for an update, the n-a3f2 freeze of a criterion
/// under an approved requirement. The only nodes read besides the changed one
/// are the requirements (n-f921).
fn check_criterion(before: &dyn Before, after: &Node) -> Result<()> {
    let previous = before.get(after.id());
    if satisfied(after) && !satisfied_of(previous.as_ref()) {
        let requirements = before.of_kind(NodeKind::Requirement);
        if !covered(&requirements, after.id()) {
            return Err(Error::invalid(refusal(&requirements, after)));
        }
    }
    if text_changed(previous.as_ref(), after) {
        let requirements = before.of_kind(NodeKind::Requirement);
        if let Some(request) = approved_target(&requirements, after.id()) {
            return Err(Error::invalid(target_refusal(after, request)));
        }
    }
    Ok(())
}

/// I2: an approved requirement may not change its title, body, or its targets
/// and relies-on edges while it stays approved. A state move is a different
/// change, and so is a free attribute, a scope move, or a typed reference.
fn check_frozen_requirement(before: &dyn Before, after: &Node) -> Result<()> {
    if after.state() != Some(RequirementState::Approved) {
        return Ok(());
    }
    let Some(previous) = before.get(after.id()) else {
        return Ok(());
    };
    if previous.state() != Some(RequirementState::Approved) {
        return Ok(());
    }
    if !text_changed(Some(&previous), after) && !frozen_edges_changed(&previous, after) {
        return Ok(());
    }
    Err(Error::invalid(format!(
        "{} is approved; req revise {} --reason … --source … first",
        after.id(),
        after.id()
    )))
}

/// Whether an update changed the title or the body. A created node has no
/// previous state, so it never counts.
fn text_changed(before: Option<&Node>, after: &Node) -> bool {
    before.is_some_and(|prev| prev.title() != after.title() || prev.body() != after.body())
}

/// Whether an update added or dropped a targets or relies-on edge.
fn frozen_edges_changed(before: &Node, after: &Node) -> bool {
    frozen_edges(before) != frozen_edges(after)
}

/// The targets and relies-on edges the freeze holds, in a comparable order.
/// Only the from side stores an edge, so the reverse side is absent.
fn frozen_edges(node: &Node) -> Vec<(&'static str, String)> {
    let mut edges: Vec<(&'static str, String)> = node
        .links()
        .iter()
        .filter(|edge| {
            !edge.reversed && matches!(edge.label, Relation::Targets | Relation::ReliesOn)
        })
        .map(|edge| (edge.label.name(), edge.to.to_string()))
        .collect();
    edges.sort();
    edges
}

/// The approved requirement that targets `criterion`, if any: the one the
/// freeze reports (n-a3f2 2). A done requirement does not freeze.
fn approved_target<'a>(nodes: &'a [Node], criterion: &NodeId) -> Option<&'a Node> {
    nodes
        .iter()
        .find(|node| node.state() == Some(RequirementState::Approved) && targets(node, criterion))
}

/// The message a criterion edit the freeze refuses reads: which requirement
/// holds it and the one next step.
fn target_refusal(criterion: &Node, request: &Node) -> String {
    format!(
        "{} is targeted by approved {}; req revise {} --reason … --source … first",
        criterion.id(),
        request.id(),
        request.id()
    )
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
