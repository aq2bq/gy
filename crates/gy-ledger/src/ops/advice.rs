//! What a node still lacks and which command could follow (N-39). One
//! derivation from the typed model, shared by every write's outcome and by
//! show, so the two cannot drift.
use crate::model::{
    Criterion, Node, NodeData, NodeId, NodeKind, Relation, Requirement, RequirementState, covered,
    filed_target,
};

/// The gaps a node still has, as short phrases. A node that is already closed,
/// approved, satisfied, or done reports none: its gaps are settled.
pub fn missing(node: &Node, all: &[Node]) -> Vec<String> {
    match node.data() {
        NodeData::Need(data) => {
            if data.closed.is_none() {
                need_missing(node)
            } else {
                closed_need_missing(node, all)
            }
        }
        NodeData::Question(data) if data.closure.is_none() => question_missing(node, all),
        NodeData::Decision(_) => decision_missing(node, all),
        NodeData::Requirement(data) if pre_approval(data) => requirement_missing(node),
        NodeData::Criterion(data) if !data.satisfied => criterion_missing(node, all),
        _ => Vec::new(),
    }
}

/// The commands that could follow. The id this node is known by is filled in;
/// the arguments that depend on a choice stay as `<...>` holes.
pub fn next(node: &Node, all: &[Node]) -> Vec<String> {
    let id = node.id();
    let mut out = match node.data() {
        NodeData::Need(data) => {
            if data.closed.is_none() {
                vec![format!("req add \"<title>\" --need {id} …")]
            } else {
                closed_need_next(node, all)
            }
        }
        NodeData::Question(data) if data.closure.is_none() => vec![
            format!("question close {id} --by … --evidence …"),
            format!("decide … --closes {id}"),
            format!("link <need> waits-on {id}"),
        ],
        NodeData::Decision(_) => decision_next(node, all),
        NodeData::Requirement(data) => requirement_next(id, data),
        NodeData::Criterion(data) => criterion_next(node, data, all),
        _ => Vec::new(),
    };
    // One rule for every kind: when the body is still empty and the state
    // reports it, the edit that would fill it leads what else could follow.
    if body_gap(node).is_some() {
        out.insert(0, format!("edit {id} --body-file … --reason …"));
    }
    out
}

fn need_missing(node: &Node) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    if linked(node, Relation::FiledAs).is_empty() {
        out.push("a filed-as requirement".to_string());
    }
    out
}

/// The unmet criteria a closed need still targets that no open need bears: the
/// gaps nobody can work anymore. A closed need is its own bearer, so this is
/// exactly the orphaned ones among its targets.
fn closed_need_missing(node: &Node, all: &[Node]) -> Vec<String> {
    orphaned_criteria(node, all)
        .into_iter()
        .map(|criterion| format!("unmet criterion {}", criterion.id()))
        .collect()
}

/// The commands that would close the gaps `closed_need_missing` reports.
fn closed_need_next(node: &Node, all: &[Node]) -> Vec<String> {
    orphaned_criteria(node, all)
        .into_iter()
        .flat_map(|criterion| satisfy_step(criterion, all))
        .collect()
}

/// The criteria a need targets that are unmet and whose every bearing need is
/// closed.
fn orphaned_criteria<'a>(need: &Node, all: &'a [Node]) -> Vec<&'a Node> {
    linked(need, Relation::Targets)
        .into_iter()
        .filter_map(|id| find(all, &id))
        .filter(|criterion| unmet_orphaned(criterion, all))
        .collect()
}

/// Whether a criterion is unsatisfied and every need that targets it is
/// closed, so no open need bears the gap. A criterion no need targets is not
/// orphaned: the bearer-less gap is reported where criteria are looked at.
pub fn unmet_orphaned(criterion: &Node, all: &[Node]) -> bool {
    if !unsatisfied(criterion) {
        return false;
    }
    let bearers: Vec<&Node> = all
        .iter()
        .filter(|node| node.kind() == NodeKind::Need)
        .filter(|need| linked(need, Relation::Targets).contains(criterion.id()))
        .collect();
    !bearers.is_empty() && bearers.iter().all(|need| closed_need(need))
}

fn closed_need(node: &Node) -> bool {
    matches!(node.data(), NodeData::Need(data) if data.closed.is_some())
}

/// A question's gaps: its body, and the need or requirement that asks it. An
/// open question nobody waits on is the one that ages into "why did I ask
/// this" (n-fa11, d-09b6).
fn question_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    if unwaited(node, all) {
        out.push(
            "a need that waits on it (waits-on) or a requirement that raised it (raised)"
                .to_string(),
        );
    }
    out
}

/// Whether nobody asks this open question: no node points a `waits-on` or a
/// `raised` edge at it. The one judgement handover, now, and list share.
pub fn unwaited(node: &Node, all: &[Node]) -> bool {
    matches!(node.data(), NodeData::Question(data) if data.closure.is_none())
        && !all.iter().any(|other| {
            linked(other, Relation::WaitsOn).contains(node.id())
                || linked(other, Relation::Raised).contains(node.id())
        })
}

fn decision_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    let closes = all
        .iter()
        .any(|other| linked(other, Relation::Closes).contains(node.id()));
    if !closes {
        out.push("a question it closes".to_string());
    }
    out
}

fn requirement_missing(node: &Node) -> Vec<String> {
    let mut out = Vec::new();
    if linked(node, Relation::ReliesOn).is_empty() {
        out.push("a relies-on decision".to_string());
    }
    if linked(node, Relation::Targets).is_empty() {
        out.push("a targets criterion".to_string());
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
        vec![format!(
            "link {} narrows <older D> --mark <text>",
            node.id()
        )]
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
        return satisfy_step(node, all);
    }
    open_sibling(node.id(), all).map_or_else(Vec::new, |sibling| satisfy_step(sibling, all))
}

/// A criterion's gaps: its body, and the coverage the satisfy needs (n-f921).
fn criterion_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    if !covered(all, node.id()) {
        out.push("an approved requirement (targets)".to_string());
    }
    out
}

/// The command that would satisfy `criterion`: the satisfy itself when an
/// approved or done requirement covers it, otherwise the requirement step that
/// must come first (n-f921).
fn satisfy_step(criterion: &Node, all: &[Node]) -> Vec<String> {
    if covered(all, criterion.id()) {
        return vec![format!("criterion satisfy {} --evidence …", criterion.id())];
    }
    if let Some(request) = filed_target(all, criterion.id()) {
        return vec![format!(
            "req approve {} --design … --heard-by … --evidence …",
            request.id()
        )];
    }
    vec![format!(
        "req add \"<title>\" --need {} --targets {}",
        bearing_need(criterion.id(), all),
        criterion.id()
    )]
}

/// The need a requirement for `criterion` would file against: an open one that
/// targets it, or the hole `<N>` when none does (a closed need takes no new
/// requirement, n-f921).
fn bearing_need(criterion: &NodeId, all: &[Node]) -> String {
    all.iter()
        .filter(|node| node.kind() == NodeKind::Need && !closed_need(node))
        .find(|need| linked(need, Relation::Targets).contains(criterion))
        .map_or_else(|| "<N>".to_string(), |need| need.id().to_string())
}

/// Another unsatisfied criterion on a need that also targets `criterion`.
fn open_sibling<'a>(criterion: &NodeId, all: &'a [Node]) -> Option<&'a Node> {
    all.iter()
        .filter(|need| need.kind() == NodeKind::Need)
        .find_map(|need| {
            linked(need, Relation::Targets)
                .into_iter()
                .filter(|id| id != criterion)
                .find_map(|id| find(all, &id))
                .filter(|node| unsatisfied(node))
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

/// The body gap a node still has: its state reports one and its body is empty.
/// One derivation, so `missing` and `next` cannot drift.
fn body_gap(node: &Node) -> Option<String> {
    if !node.body().trim().is_empty() {
        return None;
    }
    match node.data() {
        NodeData::Need(data) if data.closed.is_none() => Some("a body".to_string()),
        NodeData::Question(data) if data.closure.is_none() => {
            Some("a body (the ground for the options)".to_string())
        }
        NodeData::Decision(_) => Some("a body".to_string()),
        NodeData::Criterion(data) if !data.satisfied => Some("a body (how to measure)".to_string()),
        _ => None,
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
