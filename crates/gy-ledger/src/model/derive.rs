//! The derived judgements about the graph (n-b4ec): a need's state and
//! readiness, whether a question is open, a criterion satisfied, a requirement
//! in progress, a need closed by hand, whether nobody waits on a question,
//! which requirement covers a criterion, and which criteria are orphaned.
//! Nothing is stored; the answer is recomputed from the edges each time (D-28,
//! D-75). Each meaning has one definition, so `views` and `ops` call the same
//! function and the copies that drifted (n-f60a) cannot come back.
use super::{Node, NodeData, NodeId, NodeKind, Relation, RequirementState};

/// A need's derived state (D-28): open, closed by hand, or done (see
/// `need_state`).
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

/// Whether a question is still open; only an open question can wait on a person
/// (d-995c, n-688a).
pub fn question_open(data: &NodeData) -> bool {
    matches!(data, NodeData::Question(question) if question.closure.is_none())
}

/// Whether a criterion is satisfied.
pub fn criterion_satisfied(data: &NodeData) -> bool {
    matches!(data, NodeData::Criterion(criterion) if criterion.satisfied)
}

/// Whether a node is an unsatisfied criterion.
pub fn criterion_unsatisfied(data: &NodeData) -> bool {
    matches!(data, NodeData::Criterion(criterion) if !criterion.satisfied)
}

/// Whether a requirement is still being built: filed or approved.
pub fn requirement_in_progress(data: &NodeData) -> bool {
    matches!(data, NodeData::Requirement(requirement) if in_progress(requirement.state))
}

/// Whether a requirement state is still being built: filed or approved.
fn in_progress(state: RequirementState) -> bool {
    matches!(state, RequirementState::Filed | RequirementState::Approved)
}

/// Whether a need is closed through `need close`, as opposed to a derived
/// `done` (d-85c6).
pub fn need_closed(data: &NodeData) -> bool {
    matches!(data, NodeData::Need(need) if need.closed.is_some())
}

/// The derived state of one need, looking up its filed requirements and
/// targeted criteria in `all`. Done needs at least one filed requirement done,
/// no filed requirement still `filed` or `approved`, and every criterion it
/// targets satisfied; a `cancelled` requirement counts on neither side
/// (n-f60a, d-85c6, d-bf90).
pub fn need_state(need: &Node, all: &[Node]) -> NeedState {
    if need_closed(need.data()) {
        return NeedState::Closed;
    }
    let states: Vec<Option<RequirementState>> = edges(need, Relation::FiledAs)
        .iter()
        .map(|id| find(all, id).and_then(Node::state))
        .collect();
    let done = states.contains(&Some(RequirementState::Done))
        && !states
            .iter()
            .any(|state| matches!(state, Some(s) if in_progress(*s)))
        && criteria_satisfied(need, all);
    if done {
        NeedState::Done
    } else {
        NeedState::Open
    }
}

/// Whether every criterion the need targets is satisfied. No targets is
/// vacuously true, so a need that points at none derives as before.
fn criteria_satisfied(need: &Node, all: &[Node]) -> bool {
    edges(need, Relation::Targets)
        .iter()
        .all(|id| find(all, id).is_some_and(|node| criterion_satisfied(node.data())))
}

/// Whether a need is ready to work (the `next` condition): open, every
/// depends-on need is closed or done, and everything it `waits-on` is settled.
pub fn ready(need: &Node, all: &[Node]) -> bool {
    if need_state(need, all) != NeedState::Open {
        return false;
    }
    let dependencies_done = edges(need, Relation::DependsOn)
        .iter()
        .all(|id| find(all, id).is_some_and(|node| need_state(node, all) != NeedState::Open));
    let waits_done = edges(need, Relation::WaitsOn)
        .iter()
        .all(|id| find(all, id).is_some_and(|node| settled(node.data())));
    dependencies_done && waits_done
}

/// A question settles when it closes; a requirement settles when it is done or
/// cancelled (d-63f8).
fn settled(data: &NodeData) -> bool {
    match data {
        NodeData::Question(_) => !question_open(data),
        NodeData::Requirement(requirement) => matches!(
            requirement.state,
            RequirementState::Done | RequirementState::Cancelled
        ),
        _ => false,
    }
}

/// Whether nobody asks this open question: no node points a `waits-on` or a
/// `raised` edge at it. The one judgement handover, now, and list share
/// (n-fa11, d-09b6).
pub fn unwaited(node: &Node, all: &[Node]) -> bool {
    question_open(node.data())
        && !all.iter().any(|other| {
            edges(other, Relation::WaitsOn).contains(node.id())
                || edges(other, Relation::Raised).contains(node.id())
        })
}

/// The one coverage judgement (n-f921): whether an approved or done
/// requirement targets `criterion`. The rule (C1) and the advice (V2) share it.
pub fn covered(nodes: &[Node], criterion: &NodeId) -> bool {
    covering(nodes, criterion).is_some()
}

/// The approved or done requirement that targets `criterion`, if any.
fn covering<'a>(nodes: &'a [Node], criterion: &NodeId) -> Option<&'a Node> {
    nodes
        .iter()
        .find(|node| is_covering(node) && targets(node, criterion))
}

/// The filed requirement that targets `criterion`, if any: the one an approve
/// would turn into coverage (n-f921).
pub fn filed_target<'a>(nodes: &'a [Node], criterion: &NodeId) -> Option<&'a Node> {
    nodes
        .iter()
        .find(|node| node.state() == Some(RequirementState::Filed) && targets(node, criterion))
}

/// Whether a node carries coverage: an approved or done requirement.
fn is_covering(node: &Node) -> bool {
    matches!(
        node.state(),
        Some(RequirementState::Approved | RequirementState::Done)
    )
}

/// Whether `node` targets `criterion`.
fn targets(node: &Node, criterion: &NodeId) -> bool {
    edges(node, Relation::Targets).contains(criterion)
}

/// Needs that carry a Targets edge to a criterion, derived rather than stored
/// (C4).
pub fn bearer_count(needs: &[Node], criterion: &NodeId) -> usize {
    needs.iter().filter(|node| targets(node, criterion)).count()
}

/// Whether a criterion is unsatisfied and every need that targets it is closed,
/// so no open need bears the gap. A criterion no need targets is not orphaned:
/// the bearer-less gap is reported where criteria are looked at (C4).
pub fn unmet_orphaned(criterion: &Node, all: &[Node]) -> bool {
    if !criterion_unsatisfied(criterion.data()) {
        return false;
    }
    let bearers: Vec<&Node> = all
        .iter()
        .filter(|node| node.kind() == NodeKind::Need)
        .filter(|need| targets(need, criterion.id()))
        .collect();
    !bearers.is_empty() && bearers.iter().all(|need| need_closed(need.data()))
}

/// The ids this node points at with one relation. One function for the views,
/// the ops, and the model's own rules.
pub fn edges(node: &Node, relation: Relation) -> Vec<NodeId> {
    node.links()
        .iter()
        .filter(|edge| edge.label == relation)
        .map(|edge| edge.to.clone())
        .collect()
}

/// A node by id among an already-loaded set.
pub fn find<'a>(all: &'a [Node], id: &NodeId) -> Option<&'a Node> {
    all.iter().find(|node| node.id() == id)
}

/// A requirement's outward reference, if the node is one. One reader for the
/// views, the advice, and the id resolution.
pub fn reference(node: &Node) -> Option<&str> {
    match node.data() {
        NodeData::Requirement(requirement) => {
            requirement.reference.as_ref().map(|item| item.0.as_str())
        }
        _ => None,
    }
}
