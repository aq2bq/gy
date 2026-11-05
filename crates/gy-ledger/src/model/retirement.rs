//! Retirement preserves a closed component of the graph, with no active work.
use super::{
    Before, Change, CriterionState, NeedState, Node, NodeData, NodeId, NodeKind, criterion_state,
    need_state, question_open, requirement_in_progress,
};
use crate::store::{Error, Result};
use std::collections::BTreeSet;

pub(super) fn admit(before: &dyn Before, changes: &[Change]) -> Result<()> {
    for change in changes {
        match change {
            Change::Retired { nodes, revoke } => {
                if changes.len() != 1 {
                    return Err(Error::invalid("retire is one intent per transaction"));
                }
                check_retirement(before, nodes, *revoke)?;
            }
            Change::Created(node) | Change::Updated(node) => {
                check_write(before, node)?;
            }
            Change::Deleted(id) => {
                check_id(before, id)?;
            }
            Change::ScopeRenamed { from, .. } => check_scope(before, from)?,
        }
    }
    Ok(())
}
fn check_id(before: &dyn Before, id: &NodeId) -> Result<()> {
    if before.get(id).is_some_and(|node| node.retired().is_some()) {
        return Err(Error::invalid(format!(
            "{id} is retired; gy retire {id} --revoke --reason … first"
        )));
    }
    Ok(())
}
fn check_write(before: &dyn Before, node: &Node) -> Result<()> {
    if node.retired().is_some() {
        return Err(Error::invalid(format!(
            "{} is retired; use gy retire to change retirement",
            node.id()
        )));
    }
    check_id(before, node.id())?;
    for edge in node.links() {
        check_id(before, &edge.to)?;
    }
    Ok(())
}
fn active(node: &Node, all: &[Node]) -> bool {
    match node.data() {
        NodeData::Need(_) => need_state(node, all) == NeedState::Open,
        NodeData::Question(_) => question_open(node.data()),
        NodeData::Requirement(_) => requirement_in_progress(node.data()),
        NodeData::Criterion(_) => criterion_state(node, all) == CriterionState::Unsatisfied,
        NodeData::Decision(_) => false,
    }
}
fn check_retirement(before: &dyn Before, ids: &[NodeId], revoke: bool) -> Result<()> {
    if ids.is_empty() {
        return Err(Error::invalid("retire needs at least one ID"));
    }
    let mut all = Vec::new();
    for kind in NodeKind::ALL {
        all.extend(before.of_kind(kind));
    }
    let chosen: BTreeSet<String> = ids.iter().map(ToString::to_string).collect();
    let mut errors = Vec::new();
    for id in ids {
        match all.iter().find(|node| node.id() == id) {
            None => errors.push(format!("{id} does not exist")),
            Some(node) if node.retired().is_some() != revoke => errors.push(format!(
                "{id} is {}retired",
                if revoke { "not " } else { "already " }
            )),
            Some(node) if !revoke && active(node, &all) => {
                errors.push(format!("{id} is in progress"))
            }
            _ => {}
        }
    }
    let retired_after = |node: &Node| {
        if chosen.contains(&node.id().to_string()) {
            !revoke
        } else {
            node.retired().is_some()
        }
    };
    errors.extend(boundaries(&all, retired_after));
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::invalid(errors.join("\n")))
    }
}

fn check_scope(before: &dyn Before, from: &str) -> Result<()> {
    for kind in NodeKind::ALL {
        if let Some(node) = before
            .of_kind(kind)
            .iter()
            .find(|node| node.scope() == from && node.retired().is_some())
        {
            return Err(Error::invalid(format!(
                "{} is retired; revoke retirement before renaming its scope",
                node.id()
            )));
        }
    }
    Ok(())
}
fn boundaries(all: &[Node], retired_after: impl Fn(&Node) -> bool) -> Vec<String> {
    let index: std::collections::BTreeMap<String, &Node> =
        all.iter().map(|n| (n.id().to_string(), n)).collect();
    let mut errors = Vec::new();
    for from in all {
        for edge in from.links() {
            let Some(to) = index.get(&edge.to.to_string()) else {
                continue;
            };
            if retired_after(from) != retired_after(to) {
                errors.push(format!(
                    "remaining edge: {} {} {}",
                    from.id(),
                    edge.label.name(),
                    to.id()
                ));
            }
        }
    }
    errors
}
