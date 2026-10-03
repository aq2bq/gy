//! The write-time rule the gate runs (n-557f, n-f921, n-a3f2, d-ee34). I1: a
//! criterion may be recorded satisfied only while an approved or done
//! requirement targets it. I2: while a requirement stays approved, its title,
//! body, and its targets and relies-on edges are frozen, and so are the title
//! and body of a criterion it targets; `req revise` first. A closed need may
//! not take a new filed-as requirement. The coverage judgement is one function,
//! shared with the advice (n-f921).
use super::derive::{covered, criterion_satisfied, edges, filed_target, need_closed};
use super::{Node, NodeId, NodeKind, Relation, RequirementState};
use crate::store::{Error, Result};
use std::collections::BTreeSet;

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

/// The rule: I1 for a criterion turning satisfied, the n-a3f2 freeze for an
/// approved requirement and the criteria it targets, the shape of the
/// depends-on and lineage graphs (n-e299), a closed need taking a new filed-as
/// requirement (d-ee34), and a duplicate edge or a non-decision closes
/// (d-fb49). Every other change is admitted.
pub fn admit(before: &dyn Before, changes: &[Change]) -> Result<()> {
    if reads_state(changes) {
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
    }
    super::shape::admit(before, changes)?;
    super::edges::admit(before, changes)?;
    check_closed_need(before, changes)
}

/// Whether the transaction carries a criterion or requirement the coverage and
/// freeze rules read. Other transactions skip those two rules; the graph rule
/// (n-e299) reads only the changed needs and decisions, and only when they add
/// an edge, so a commit still decodes nothing it does not judge (n-f921,
/// n-a3f2).
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
    let was_satisfied = previous
        .as_ref()
        .is_some_and(|node| criterion_satisfied(node.data()));
    if criterion_satisfied(after.data()) && !was_satisfied {
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
    let mut frozen = Vec::new();
    for relation in [Relation::Targets, Relation::ReliesOn] {
        for id in edges(node, relation) {
            frozen.push((relation.name(), id.to_string()));
        }
    }
    frozen.sort();
    frozen
}

/// The approved requirement that targets `criterion`, if any: the one the
/// freeze reports (n-a3f2 2). A done requirement does not freeze.
fn approved_target<'a>(nodes: &'a [Node], criterion: &NodeId) -> Option<&'a Node> {
    nodes.iter().find(|node| {
        node.state() == Some(RequirementState::Approved)
            && edges(node, Relation::Targets).contains(criterion)
    })
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

/// d-ee34: a need that has closed may not take a new filed-as requirement.
/// Only an updated need whose filed-as edges grew is read; a transaction that
/// adds none reads no previous state, and a need that already held one when it
/// closed keeps it (the gate looks at what the write changes).
fn check_closed_need(before: &dyn Before, changes: &[Change]) -> Result<()> {
    for change in changes {
        let Change::Updated(node) = change else {
            continue;
        };
        if !need_closed(node.data()) {
            continue;
        }
        let after = filed_as(node);
        if after.is_empty() {
            continue;
        }
        let previous = before
            .get(node.id())
            .map(|node| filed_as(&node))
            .unwrap_or_default();
        if after.difference(&previous).next().is_some() {
            return Err(Error::invalid(format!("{} is closed", node.id())));
        }
    }
    Ok(())
}

/// The filed-as edges of a need, in a comparable order. Only the from side
/// stores an edge, so the reverse side is absent.
fn filed_as(node: &Node) -> BTreeSet<String> {
    edges(node, Relation::FiledAs)
        .iter()
        .map(|id| id.to_string())
        .collect()
}
