//! What a node still lacks and which command could follow (N-39). One
//! derivation from the typed model, shared by every write's outcome and by
//! show, so the two cannot drift.
mod need;

use self::need::{need_missing, unmet_criteria};
use crate::model::{
    Node, NodeData, NodeId, NodeKind, Relation, Requirement, RequirementState, covered,
    criterion_unsatisfied, edges, filed_target, find, need_closed, question_open, reference,
    unmet_orphaned, unwaited,
};

/// The gaps a node still has, as short phrases. A node that is already closed,
/// approved, satisfied, or done reports none: its gaps are settled.
pub fn missing(node: &Node, all: &[Node]) -> Vec<String> {
    match node.data() {
        NodeData::Need(_) if !need_closed(node.data()) => need_missing(node, all),
        NodeData::Need(_) => closed_need_missing(node, all),
        NodeData::Question(_) if question_open(node.data()) => question_missing(node, all),
        NodeData::Decision(_) => decision_missing(node),
        NodeData::Requirement(data) if pre_approval(data) => requirement_missing(node),
        NodeData::Criterion(_) if criterion_unsatisfied(node.data()) => {
            criterion_missing(node, all)
        }
        _ => Vec::new(),
    }
}

/// The commands that could follow. The id this node is known by is filled in;
/// the arguments that depend on a choice stay as `<...>` holes.
pub fn next(node: &Node, all: &[Node]) -> Vec<String> {
    let id = node.id();
    let mut out = match node.data() {
        NodeData::Need(_) if !need_closed(node.data()) => {
            let mut out = vec![format!("req add \"<title>\" --need {id} …")];
            if !edges(node, Relation::FiledAs).is_empty() {
                out.extend(
                    unmet_criteria(node, all)
                        .into_iter()
                        .flat_map(|criterion| satisfy_step(criterion, all)),
                );
            }
            out
        }
        NodeData::Need(_) => closed_need_next(node, all),
        NodeData::Question(_) if question_open(node.data()) => vec![
            format!("question close {id} --by … --evidence …"),
            format!("decide … --closes {id}"),
            format!("link <need> waits-on {id}"),
        ],
        NodeData::Decision(_) => decision_next(node, all),
        NodeData::Requirement(data) => requirement_next(id, data),
        NodeData::Criterion(_) => criterion_next(node, all),
        _ => Vec::new(),
    };
    // One rule for every kind: when the body is still empty and the state
    // reports it, the edit that would fill it leads what else could follow.
    if body_gap(node).is_some() {
        out.insert(0, format!("edit {id} --body-file … --reason …"));
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
    edges(need, Relation::Targets)
        .iter()
        .filter_map(|id| find(all, id))
        .filter(|criterion| unmet_orphaned(criterion, all))
        .collect()
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

/// A decision's gaps: its body. A decision that closes no question is the
/// normal shape (d-43f624); the question side asks for its closer.
fn decision_missing(node: &Node) -> Vec<String> {
    body_gap(node).into_iter().collect()
}

fn requirement_missing(node: &Node) -> Vec<String> {
    let mut out = Vec::new();
    if edges(node, Relation::ReliesOn).is_empty() {
        out.push("a relies-on decision".to_string());
    }
    if edges(node, Relation::Targets).is_empty() {
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

fn criterion_next(node: &Node, all: &[Node]) -> Vec<String> {
    if criterion_unsatisfied(node.data()) {
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
        .filter(|node| node.kind() == NodeKind::Need && !need_closed(node.data()))
        .find(|need| edges(need, Relation::Targets).contains(criterion))
        .map_or_else(|| "<N>".to_string(), |need| need.id().to_string())
}

/// Another unsatisfied criterion on a need that also targets `criterion`: a
/// satisfied sibling early in the list does not hide an unsatisfied one behind.
fn open_sibling<'a>(criterion: &NodeId, all: &'a [Node]) -> Option<&'a Node> {
    all.iter()
        .filter(|need| need.kind() == NodeKind::Need)
        .filter(|need| edges(need, Relation::Targets).contains(criterion))
        .find_map(|need| {
            edges(need, Relation::Targets)
                .into_iter()
                .filter(|id| id != criterion)
                .filter_map(|id| find(all, &id))
                .find(|node| criterion_unsatisfied(node.data()))
        })
}

/// A requirement still being filed: not approved yet. A revised requirement
/// keeps its approval, so its earlier gaps are not raised again.
fn pre_approval(data: &Requirement) -> bool {
    data.state == RequirementState::Filed && data.approval.is_none()
}

/// The body gap a node still has: its state reports one and its body is empty.
/// One derivation, so `missing` and `next` cannot drift.
fn body_gap(node: &Node) -> Option<String> {
    if !node.body().trim().is_empty() {
        return None;
    }
    match node.data() {
        NodeData::Need(_) if !need_closed(node.data()) => Some("a body".to_string()),
        NodeData::Question(_) if question_open(node.data()) => {
            Some("a body (the ground for the options)".to_string())
        }
        NodeData::Decision(_) => Some("a body".to_string()),
        NodeData::Criterion(_) if criterion_unsatisfied(node.data()) => {
            Some("a body (how to measure)".to_string())
        }
        _ => None,
    }
}
