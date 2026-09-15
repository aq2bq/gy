//! Turning intermediate 0.4 nodes into gy-ledger nodes (N-67): the kind
//! mapping, the old id as an alias, and the requirement reference.
use crate::legacy::LegacyNode;
use gy_ledger::{
    Alias, Closed, ClosedBy, Closure, DecisionScope, Error, Node, NodeData, NodeId, NodeKind, Ref,
    RequirementState, Result,
};

/// The gy-ledger kind a new node uses.
pub fn kind(name: &str) -> Result<NodeKind> {
    NodeKind::ALL
        .into_iter()
        .find(|kind| kind.name() == name)
        .ok_or_else(|| Error::invalid(format!("unknown kind {name}")))
}

/// Build one new node: the kind's typed fields, the shared body and free
/// attributes, and the old id as an alias. `derive` tells whether a need's
/// closed state is left to derivation from its requirements.
pub fn build(
    legacy: &LegacyNode,
    id: NodeId,
    ref_base: Option<&str>,
    derive: bool,
) -> Result<Node> {
    let mut node = match legacy.new_kind() {
        "need" => need(legacy, id, derive)?,
        "question" => question(legacy, id)?,
        "decision" => decision(legacy, id)?,
        "requirement" => requirement(legacy, id, ref_base)?,
        "criterion" => criterion(legacy, id)?,
        other => return Err(Error::invalid(format!("unknown kind {other}"))),
    };
    node.set_body(body(legacy));
    free(&mut node, legacy);
    node.add_alias(Alias(legacy.id.clone()));
    Ok(node)
}

fn body(legacy: &LegacyNode) -> String {
    if legacy.kind == "gate" {
        format!("gate から移行\n{}", legacy.body)
    } else {
        legacy.body.clone()
    }
}

fn need(legacy: &LegacyNode, id: NodeId, derive: bool) -> Result<Node> {
    let mut node = Node::need(id, &legacy.scope, &legacy.created, &legacy.title)?;
    if !derive {
        if let Some(closed) = closed(legacy) {
            if let NodeData::Need(data) = node.data_mut() {
                data.closed = Some(closed);
            }
        }
    }
    Ok(node)
}

fn closed(legacy: &LegacyNode) -> Option<Closed> {
    if legacy.status() != "complete" {
        return None;
    }
    let evidence = match (legacy.dropped_reason(), legacy.dropped_by()) {
        (Some(reason), Some(by)) => format!("{reason} ({by})"),
        (Some(reason), None) => reason.to_string(),
        _ => "migrated: complete".to_string(),
    };
    Some(Closed {
        by: ClosedBy::Fact,
        evidence,
    })
}

fn question(legacy: &LegacyNode, id: NodeId) -> Result<Node> {
    let mut node = Node::question(id, &legacy.scope, &legacy.created, &legacy.title)?;
    if let NodeData::Question(data) = node.data_mut() {
        data.decider = legacy.decider().map(str::to_string);
        data.options = legacy.options().to_vec();
        data.closure = legacy.closed_by().and_then(closure);
        data.evidence = legacy
            .closure_note()
            .or(legacy.closed_at())
            .map(str::to_string);
    }
    Ok(node)
}

fn closure(name: &str) -> Option<Closure> {
    match name {
        "fact" => Some(Closure::Fact),
        "decision" => Some(Closure::Decision),
        "non-decision" => Some(Closure::NonDecision),
        _ => None,
    }
}

fn decision(legacy: &LegacyNode, id: NodeId) -> Result<Node> {
    let scope = match legacy.decision_scope() {
        Some(text) => DecisionScope::recorded(text)?,
        None => DecisionScope::migration_unrecorded(),
    };
    Node::decision(id, &legacy.scope, &legacy.created, &legacy.title, scope)
}

fn requirement(legacy: &LegacyNode, id: NodeId, ref_base: Option<&str>) -> Result<Node> {
    let mut node = Node::requirement(
        id,
        &legacy.scope,
        &legacy.created,
        &legacy.title,
        state(legacy.status()),
    )?;
    if let NodeData::Requirement(data) = node.data_mut() {
        data.reference = reference(legacy, ref_base);
    }
    Ok(node)
}

/// The 0.4 requirement states, reduced to the four of D-70.
fn state(status: &str) -> RequirementState {
    match status {
        "awaiting-implementation"
        | "awaiting-audit"
        | "awaiting-pr"
        | "awaiting-merge"
        | "awaiting-production"
        | "awaiting-cleanup" => RequirementState::Approved,
        "complete" => RequirementState::Done,
        _ => RequirementState::Filed,
    }
}

/// A requirement's `#N` id becomes its outward reference, with the optional
/// prefix (D-73).
fn reference(legacy: &LegacyNode, ref_base: Option<&str>) -> Option<Ref> {
    let number = legacy.id.strip_prefix('#')?;
    Some(Ref(match ref_base {
        Some(base) => format!("{base}{number}"),
        None => legacy.id.clone(),
    }))
}

fn criterion(legacy: &LegacyNode, id: NodeId) -> Result<Node> {
    let mut node = Node::criterion(id, &legacy.scope, &legacy.created, &legacy.title)?;
    if let NodeData::Criterion(data) = node.data_mut() {
        data.satisfied = legacy.satisfied();
        data.evidence = legacy.evidence().map(str::to_string);
        data.satisfied_at = legacy.satisfied_at().map(str::to_string);
    }
    Ok(node)
}

fn free(node: &mut Node, legacy: &LegacyNode) {
    for (name, value) in [
        ("next_evidence", legacy.next_evidence()),
        ("responsible", legacy.responsible()),
        ("remaining_work", legacy.remaining_work()),
        ("residual", legacy.residual()),
        ("unresolved", legacy.unresolved()),
        ("belongs-to", legacy.belongs_to()),
    ] {
        if let Some(value) = value {
            node.set_free(name, value);
        }
    }
    if legacy.new_kind() == "requirement" {
        if let Some(evidence) = legacy.evidence() {
            node.set_free("evidence", evidence);
        }
    }
}
