//! What a node still lacks and which command could follow (N-39). One
//! derivation from the typed model, shared by every write's outcome and by
//! show, so the two cannot drift.
use crate::model::{
    Criterion, Node, NodeData, NodeId, NodeKind, Relation, Requirement, RequirementState,
};

/// The gaps a node still has, as short phrases. A node that is already closed,
/// approved, satisfied, or done reports none: its gaps are settled.
pub fn missing(node: &Node, all: &[Node]) -> Vec<String> {
    match node.data() {
        NodeData::Need(data) if data.closed.is_none() => need_missing(node),
        NodeData::Question(data) if data.closure.is_none() => body(node, "本文（選択肢の根拠）"),
        NodeData::Decision(_) => decision_missing(node, all),
        NodeData::Requirement(data) if pre_approval(data) => requirement_missing(node),
        NodeData::Criterion(data) if !data.satisfied => body(node, "本文（測り方）"),
        _ => Vec::new(),
    }
}

/// The commands that could follow. The id this node is known by is filled in;
/// the arguments that depend on a choice stay as `<...>` holes.
pub fn next(node: &Node, all: &[Node]) -> Vec<String> {
    let id = node.id();
    match node.data() {
        NodeData::Need(data) if data.closed.is_none() => vec![
            format!("edit {id} --body-file … --reason …"),
            format!("req add \"<題>\" --need {id} …"),
        ],
        NodeData::Question(data) if data.closure.is_none() => vec![
            format!("question close {id} --by … --evidence …"),
            format!("decide … --closes {id}"),
        ],
        NodeData::Decision(_) => decision_next(node, all),
        NodeData::Requirement(data) => requirement_next(id, data),
        NodeData::Criterion(data) => criterion_next(node, data, all),
        _ => Vec::new(),
    }
}

fn need_missing(node: &Node) -> Vec<String> {
    let mut out = body(node, "本文");
    if linked(node, Relation::FiledAs).is_empty() {
        out.push("filed-as の要求".to_string());
    }
    out
}

fn decision_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out = body(node, "本文");
    let closes = all
        .iter()
        .any(|other| linked(other, Relation::Closes).contains(node.id()));
    if !closes {
        out.push("closes した論点".to_string());
    }
    out
}

fn requirement_missing(node: &Node) -> Vec<String> {
    let mut out = Vec::new();
    if linked(node, Relation::ReliesOn).is_empty() {
        out.push("relies-on の決定".to_string());
    }
    if linked(node, Relation::Targets).is_empty() {
        out.push("targets の AC".to_string());
    }
    if reference(node).is_none() {
        out.push("ref".to_string());
    }
    out
}

fn decision_next(node: &Node, all: &[Node]) -> Vec<String> {
    let lineage = node.links().iter().any(|edge| {
        matches!(
            edge.label,
            Relation::Narrows | Relation::Widens | Relation::Supersedes | Relation::Completes
        )
    });
    let sibling = all.iter().any(|other| {
        other.kind() == NodeKind::Decision
            && other.id() != node.id()
            && other.scope() == node.scope()
    });
    if !lineage && sibling {
        vec![format!("link {} narrows <古い D> --mark <文>", node.id())]
    } else {
        Vec::new()
    }
}

fn requirement_next(id: &NodeId, data: &Requirement) -> Vec<String> {
    match data.state {
        RequirementState::Filed => {
            vec![format!(
                "req approve {id} --design … --heard-by … --evidence …"
            )]
        }
        RequirementState::Approved => vec![format!("req done {id} --evidence …")],
        RequirementState::Done | RequirementState::Cancelled => Vec::new(),
    }
}

fn criterion_next(node: &Node, data: &Criterion, all: &[Node]) -> Vec<String> {
    if !data.satisfied {
        return vec![format!("criterion satisfy {} --evidence …", node.id())];
    }
    match open_sibling(node.id(), all) {
        Some(sibling) => vec![format!("criterion satisfy {sibling}")],
        None => Vec::new(),
    }
}

/// Another unsatisfied criterion on a need that also targets `criterion`.
fn open_sibling(criterion: &NodeId, all: &[Node]) -> Option<NodeId> {
    all.iter()
        .filter(|need| need.kind() == NodeKind::Need)
        .find_map(|need| {
            linked(need, Relation::Targets)
                .into_iter()
                .find(|id| id != criterion && find(all, id).is_some_and(unsatisfied))
        })
}

/// A requirement still being filed: not approved yet. A revised requirement
/// keeps its approval, so its earlier gaps are not raised again.
fn pre_approval(data: &Requirement) -> bool {
    data.state == RequirementState::Filed && data.approval.is_none()
}

fn unsatisfied(node: &Node) -> bool {
    matches!(node.data(), NodeData::Criterion(data) if !data.satisfied)
}

fn body(node: &Node, name: &str) -> Vec<String> {
    if node.body().trim().is_empty() {
        vec![name.to_string()]
    } else {
        Vec::new()
    }
}

fn linked(node: &Node, relation: Relation) -> Vec<NodeId> {
    node.links()
        .iter()
        .filter(|edge| edge.label == relation)
        .map(|edge| edge.to.clone())
        .collect()
}

fn reference(node: &Node) -> Option<&str> {
    match node.data() {
        NodeData::Requirement(data) => data.reference.as_ref().map(|item| item.0.as_str()),
        _ => None,
    }
}

fn find<'a>(all: &'a [Node], id: &NodeId) -> Option<&'a Node> {
    all.iter().find(|node| node.id() == id)
}
