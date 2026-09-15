//! Deriving a need's state and readiness from the graph (D-28, D-75). Nothing
//! is stored; the answer is recomputed from the edges each time.
use crate::model::{Node, NodeData, NodeId, NodeKind, Relation, RequirementState};

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
    let questions_done = waiting_on(need)
        .iter()
        .all(|id| find(all, id).is_some_and(question_closed));
    dependencies_done && questions_done
}

/// The free `waiting-on` attribute as ids (comma separated).
fn waiting_on(need: &Node) -> Vec<NodeId> {
    need.free("waiting-on")
        .map(|value| {
            value
                .split(',')
                .filter_map(|part| parse_id(part.trim()))
                .collect()
        })
        .unwrap_or_default()
}

/// Parse an id such as `q-3f9a` back into a `NodeId`, for ids kept in a free
/// attribute where only the text survives.
fn parse_id(text: &str) -> Option<NodeId> {
    let (prefix, hash) = text.split_once('-')?;
    let kind = NodeKind::ALL
        .into_iter()
        .find(|kind| kind.prefix() == prefix)?;
    NodeId::from_hash(kind, hash).ok()
}

fn question_closed(node: &Node) -> bool {
    matches!(node.data(), NodeData::Question(data) if data.closure.is_some())
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
