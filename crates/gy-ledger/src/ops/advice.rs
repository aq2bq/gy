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

/// The commands that could follow, with `<...>` holes for their arguments.
pub fn next(node: &Node, all: &[Node]) -> Vec<String> {
    match node.data() {
        NodeData::Need(data) if data.closed.is_none() => strings(&[
            "edit <ID> --body-file … --reason …",
            "req add \"<題>\" --need <ID> …",
        ]),
        NodeData::Question(data) if data.closure.is_none() => strings(&[
            "question close <ID> --by … --evidence …",
            "decide … --closes <ID>",
        ]),
        NodeData::Decision(_) => decision_next(node, all),
        NodeData::Requirement(data) => requirement_next(data),
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
        strings(&["link <D> narrows <古い D> --mark <文>"])
    } else {
        Vec::new()
    }
}

fn requirement_next(data: &Requirement) -> Vec<String> {
    match data.state {
        RequirementState::Filed => {
            strings(&["req approve <ID> --design … --heard-by … --evidence …"])
        }
        RequirementState::Approved => strings(&["req done <ID> --evidence …"]),
        RequirementState::Done | RequirementState::Cancelled => Vec::new(),
    }
}

fn criterion_next(node: &Node, data: &Criterion, all: &[Node]) -> Vec<String> {
    if !data.satisfied {
        return strings(&["criterion satisfy <AC> --evidence …"]);
    }
    let sibling = all
        .iter()
        .any(|need| has_open_sibling(need, node.id(), all));
    if sibling {
        strings(&["criterion satisfy <他の AC>"])
    } else {
        Vec::new()
    }
}

fn has_open_sibling(need: &Node, criterion: &NodeId, all: &[Node]) -> bool {
    need.kind() == NodeKind::Need
        && linked(need, Relation::Targets)
            .iter()
            .any(|id| id != criterion && find(all, id).is_some_and(unsatisfied))
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

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
