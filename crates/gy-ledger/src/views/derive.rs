//! Deriving a need's state and readiness from the graph (D-28, D-75). Nothing
//! is stored; the answer is recomputed from the edges each time.
use crate::model::{Node, NodeData, NodeId, Relation, RequirementState};

/// A need's derived state (D-28): open, closed by hand, or done because every
/// requirement it was filed as is done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedState {
    Open,
    Closed,
    Done,
}
impl NeedState {
    pub fn name(self) -> &'static str {
        ["open", "closed", "done"][self as usize]
    }
}

/// A requirement is in progress while it is filed or approved.
pub fn requirement_in_progress(requirement: &Node) -> bool {
    matches!(
        requirement_state(requirement),
        Some(RequirementState::Filed | RequirementState::Approved)
    )
}

/// The derived state of one need, looking up its filed requirements in `all`.
pub fn need_state(need: &Node, all: &[Node]) -> NeedState {
    if let NodeData::Need(data) = need.data() {
        if data.closed.is_some() {
            return NeedState::Closed;
        }
    }
    let filed = edges(need, Relation::FiledAs);
    let done = !filed.is_empty()
        && filed.iter().all(|id| {
            find(all, id)
                .is_some_and(|node| requirement_state(node) == Some(RequirementState::Done))
        });
    if done {
        NeedState::Done
    } else {
        NeedState::Open
    }
}

/// Whether a need is ready to work (the `next` condition): open, every
/// depends-on need is closed or done, and every question named by the free
/// `waiting-on` attribute is closed.
pub fn ready(need: &Node, all: &[Node]) -> bool {
    if need_state(need, all) != NeedState::Open {
        return false;
    }
    let dependencies_done = edges(need, Relation::DependsOn)
        .iter()
        .all(|id| find(all, id).is_some_and(|node| need_state(node, all) != NeedState::Open));
    let waits_done = edges(need, Relation::WaitsOn)
        .iter()
        .all(|id| find(all, id).is_some_and(waited_on_resolved));
    dependencies_done && waits_done
}

/// A question settles when it closes; a requirement settles when it is done or
/// cancelled (d-63f8).
fn waited_on_resolved(node: &Node) -> bool {
    match node.data() {
        NodeData::Question(data) => data.closure.is_some(),
        NodeData::Requirement(data) => {
            matches!(
                data.state,
                RequirementState::Done | RequirementState::Cancelled
            )
        }
        _ => false,
    }
}

fn requirement_state(node: &Node) -> Option<RequirementState> {
    match node.data() {
        NodeData::Requirement(data) => Some(data.state),
        _ => None,
    }
}

/// The ids this node points at with one relation.
pub(super) fn edges(node: &Node, relation: Relation) -> Vec<NodeId> {
    node.links()
        .iter()
        .filter(|edge| edge.label == relation)
        .map(|edge| edge.to.clone())
        .collect()
}

/// A node by id among an already-loaded set.
pub(super) fn find<'a>(all: &'a [Node], id: &NodeId) -> Option<&'a Node> {
    all.iter().find(|node| node.id() == id)
}

/// A requirement's outward reference, if the node is one.
pub(super) fn reference(node: &Node) -> Option<String> {
    match node.data() {
        NodeData::Requirement(data) => data.reference.as_ref().map(|ref_| ref_.0.clone()),
        _ => None,
    }
}
