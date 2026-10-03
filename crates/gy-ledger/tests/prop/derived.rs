//! Straight re-implementations of the derived judgements from the
//! architecture text (Nd3–Nd5, Q3, C3, C4, D2, V1–V5), written independently
//! of `gy-ledger`'s own functions so that a semantic change in the source
//! shows up as a mismatch (n-7a58, r-e9f2). `show`, `list`, `next`, `now` and
//! `handover` are the library's answers; these are the spec's.
use gy_ledger::{
    Narrowed, Node, NodeData, NodeId, NodeKind, Relation, RequirementState, Retraction, Warning,
};
use std::collections::BTreeSet;

/// The forward edges of a node; only the from side stores an edge.
pub fn fwd(node: &Node) -> impl Iterator<Item = &gy_ledger::Edge> {
    node.links().iter().filter(|edge| !edge.reversed)
}

pub fn find<'a>(all: &'a [Node], id: &NodeId) -> Option<&'a Node> {
    all.iter().find(|node| node.id() == id)
}

pub fn edges(node: &Node, relation: Relation) -> Vec<NodeId> {
    fwd(node)
        .filter(|edge| edge.label == relation)
        .map(|edge| edge.to.clone())
        .collect()
}

pub fn targets(node: &Node) -> Vec<NodeId> {
    edges(node, Relation::Targets)
}

pub fn filed_as(node: &Node) -> Vec<NodeId> {
    edges(node, Relation::FiledAs)
}

pub fn depends_on(node: &Node) -> Vec<NodeId> {
    edges(node, Relation::DependsOn)
}

pub fn waits_on(node: &Node) -> Vec<NodeId> {
    edges(node, Relation::WaitsOn)
}

fn relies_on(node: &Node) -> Vec<NodeId> {
    edges(node, Relation::ReliesOn)
}

fn raised(node: &Node) -> Vec<NodeId> {
    edges(node, Relation::Raised)
}

pub fn closed_by_hand(node: &Node) -> bool {
    matches!(node.data(), NodeData::Need(data) if data.closed.is_some())
}

pub fn criterion_satisfied(node: &Node) -> bool {
    matches!(node.data(), NodeData::Criterion(data) if data.satisfied)
}

fn unsatisfied(node: &Node) -> bool {
    matches!(node.data(), NodeData::Criterion(data) if !data.satisfied)
}

fn requirement_state(node: &Node) -> Option<RequirementState> {
    match node.data() {
        NodeData::Requirement(data) => Some(data.state),
        _ => None,
    }
}

/// Nd3: closed by hand; else done when at least one filed-as requirement is
/// done, none is filed or approved, and every targeted criterion is satisfied
/// (a cancelled requirement counts neither way); else open.
pub fn need_state(need: &Node, all: &[Node]) -> &'static str {
    if closed_by_hand(need) {
        return "closed";
    }
    let states: Vec<Option<RequirementState>> = filed_as(need)
        .into_iter()
        .map(|id| find(all, &id).and_then(requirement_state))
        .collect();
    let done = states.contains(&Some(RequirementState::Done))
        && !states.iter().any(|state| {
            matches!(
                state,
                Some(RequirementState::Filed | RequirementState::Approved)
            )
        })
        && targets(need)
            .into_iter()
            .all(|id| find(all, &id).is_some_and(criterion_satisfied));
    if done { "done" } else { "open" }
}

/// Nd4: a need the source calls done has every targeted criterion satisfied.
pub fn done_has_no_unmet(need: &Node, all: &[Node]) -> bool {
    targets(need)
        .into_iter()
        .all(|id| find(all, &id).is_some_and(criterion_satisfied))
}

/// Nd5: open, every depends-on need closed or done, and every waits-on node
/// settled (a question closed; a requirement done or cancelled).
pub fn ready(need: &Node, all: &[Node]) -> bool {
    need_state(need, all) == "open"
        && depends_on(need)
            .into_iter()
            .all(|id| find(all, &id).is_some_and(|node| need_state(node, all) != "open"))
        && waits_on(need)
            .into_iter()
            .all(|id| find(all, &id).is_some_and(waited_on_resolved))
}

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

/// V4: the ready needs, ordered by created then id.
pub fn ready_ordered(all: &[Node]) -> Vec<String> {
    let mut needs: Vec<&Node> = all
        .iter()
        .filter(|node| node.kind() == NodeKind::Need && ready(node, all))
        .collect();
    needs.sort_by(|a, b| {
        a.created()
            .cmp(b.created())
            .then_with(|| a.id().to_string().cmp(&b.id().to_string()))
    });
    needs.iter().map(|node| node.id().to_string()).collect()
}

/// Q3: a question is open iff it has no closure.
pub fn question_open(node: &Node) -> bool {
    matches!(node.data(), NodeData::Question(data) if data.closure.is_none())
}

/// C3: covered iff an approved or done requirement targets it.
pub fn covered(all: &[Node], criterion: &NodeId) -> bool {
    all.iter().any(|node| {
        matches!(
            node.state(),
            Some(RequirementState::Approved | RequirementState::Done)
        ) && targets(node).contains(criterion)
    })
}

/// C4: the needs that target a criterion.
pub fn bearers<'a>(all: &'a [Node], criterion: &NodeId) -> Vec<&'a Node> {
    all.iter()
        .filter(|node| node.kind() == NodeKind::Need && targets(node).contains(criterion))
        .collect()
}

/// C4: a criterion is orphaned iff unmet, borne by at least one need, and every
/// bearing need is closed through `need close`.
pub fn orphaned(criterion: &Node, all: &[Node]) -> bool {
    if !unsatisfied(criterion) {
        return false;
    }
    let bearers = bearers(all, criterion.id());
    !bearers.is_empty() && bearers.iter().all(|need| closed_by_hand(need))
}

/// V2: the gaps a node reports, by kind and state.
pub fn missing(node: &Node, all: &[Node]) -> Vec<String> {
    match node.data() {
        NodeData::Need(data) => {
            if data.closed.is_none() {
                need_missing(node, all)
            } else {
                closed_need_missing(node, all)
            }
        }
        NodeData::Question(data) if data.closure.is_none() => question_missing(node, all),
        NodeData::Decision(_) => body_gap(node).into_iter().collect(),
        NodeData::Requirement(data)
            if data.state == RequirementState::Filed && data.approval.is_none() =>
        {
            requirement_missing(node)
        }
        NodeData::Criterion(data) if !data.satisfied => criterion_missing(node, all),
        _ => Vec::new(),
    }
}

fn need_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    if has_live_filed_as(node, all) {
        out.extend(
            unmet_criteria(node, all)
                .into_iter()
                .map(|criterion| format!("unmet criterion {}", criterion.id())),
        );
    } else {
        out.push("a filed-as requirement".to_string());
    }
    out
}

fn has_live_filed_as(node: &Node, all: &[Node]) -> bool {
    filed_as(node)
        .into_iter()
        .filter_map(|id| find(all, &id))
        .any(|other| {
            matches!(
                other.data(),
                NodeData::Requirement(data) if data.state != RequirementState::Cancelled
            )
        })
}

fn unmet_criteria<'a>(need: &Node, all: &'a [Node]) -> Vec<&'a Node> {
    targets(need)
        .into_iter()
        .filter_map(|id| find(all, &id))
        .filter(|criterion| unsatisfied(criterion))
        .filter(|criterion| !in_progress_targets(all, criterion.id()))
        .collect()
}

fn in_progress_targets(all: &[Node], criterion: &NodeId) -> bool {
    all.iter().any(|node| {
        matches!(
            node.state(),
            Some(RequirementState::Filed | RequirementState::Approved)
        ) && targets(node).contains(criterion)
    })
}

fn closed_need_missing(node: &Node, all: &[Node]) -> Vec<String> {
    orphaned_criteria(node, all)
        .into_iter()
        .map(|criterion| format!("unmet criterion {}", criterion.id()))
        .collect()
}

fn orphaned_criteria<'a>(need: &Node, all: &'a [Node]) -> Vec<&'a Node> {
    targets(need)
        .into_iter()
        .filter_map(|id| find(all, &id))
        .filter(|criterion| orphaned(criterion, all))
        .collect()
}

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

fn unwaited(node: &Node, all: &[Node]) -> bool {
    question_open(node)
        && !all
            .iter()
            .any(|other| waits_on(other).contains(node.id()) || raised(other).contains(node.id()))
}

fn requirement_missing(node: &Node) -> Vec<String> {
    let mut out = Vec::new();
    if relies_on(node).is_empty() {
        out.push("a relies-on decision".to_string());
    }
    if targets(node).is_empty() {
        out.push("a targets criterion".to_string());
    }
    if reference(node).is_none() {
        out.push("ref".to_string());
    }
    out
}

fn criterion_missing(node: &Node, all: &[Node]) -> Vec<String> {
    let mut out: Vec<String> = body_gap(node).into_iter().collect();
    if !covered(all, node.id()) {
        out.push("an approved requirement (targets)".to_string());
    }
    out
}

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

fn reference(node: &Node) -> Option<&str> {
    match node.data() {
        NodeData::Requirement(data) => data.reference.as_ref().map(|item| item.0.as_str()),
        _ => None,
    }
}

/// V3: the steps `next` reports for a need or a criterion.
pub fn next(node: &Node, all: &[Node]) -> Vec<String> {
    let id = node.id();
    let mut out = match node.data() {
        NodeData::Need(data) => {
            if data.closed.is_none() {
                let mut out = vec![format!("req add \"<title>\" --need {id} …")];
                if !filed_as(node).is_empty() {
                    out.extend(
                        unmet_criteria(node, all)
                            .into_iter()
                            .flat_map(|criterion| satisfy_step(criterion, all)),
                    );
                }
                out
            } else {
                orphaned_criteria(node, all)
                    .into_iter()
                    .flat_map(|criterion| satisfy_step(criterion, all))
                    .collect()
            }
        }
        NodeData::Question(data) if data.closure.is_none() => vec![
            format!("question close {id} --by … --evidence …"),
            format!("decide … --closes {id}"),
            format!("link <need> waits-on {id}"),
        ],
        NodeData::Decision(_) => decision_next(node, all),
        NodeData::Requirement(data) => match data.state {
            RequirementState::Filed => {
                vec![format!(
                    "req approve {id} --design … --heard-by … --evidence …"
                )]
            }
            RequirementState::Approved => vec![format!("req done {id} --evidence …")],
            RequirementState::Done | RequirementState::Cancelled => Vec::new(),
        },
        NodeData::Criterion(data) => {
            if !data.satisfied {
                satisfy_step(node, all)
            } else {
                open_sibling(node.id(), all)
                    .map_or_else(Vec::new, |sibling| satisfy_step(sibling, all))
            }
        }
        _ => Vec::new(),
    };
    if body_gap(node).is_some() {
        out.insert(0, format!("edit {id} --body-file … --reason …"));
    }
    out
}

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

fn filed_target<'a>(all: &'a [Node], criterion: &NodeId) -> Option<&'a Node> {
    all.iter().find(|node| {
        node.state() == Some(RequirementState::Filed) && targets(node).contains(criterion)
    })
}

fn bearing_need(criterion: &NodeId, all: &[Node]) -> String {
    all.iter()
        .filter(|node| node.kind() == NodeKind::Need && !closed_by_hand(node))
        .find(|need| targets(need).contains(criterion))
        .map_or_else(|| "<N>".to_string(), |need| need.id().to_string())
}

fn open_sibling<'a>(criterion: &NodeId, all: &'a [Node]) -> Option<&'a Node> {
    all.iter()
        .filter(|need| need.kind() == NodeKind::Need)
        .filter(|need| targets(need).contains(criterion))
        .find_map(|need| {
            targets(need)
                .into_iter()
                .filter(|id| id != criterion)
                .filter_map(|id| find(all, &id))
                .find(|node| unsatisfied(node))
        })
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

/// D2: the retractions a node carries, from the incoming `narrows` /
/// `supersedes` edges of the whole graph.
pub fn retraction(all: &[Node], id: &str) -> Option<Retraction> {
    let mut out = Retraction::default();
    let mut any = false;
    for node in all {
        for edge in node.links() {
            if edge.to.to_string() != id {
                continue;
            }
            match edge.label {
                Relation::Supersedes => {
                    out.superseded_by.push(node.id().to_string());
                    any = true;
                }
                Relation::Narrows => {
                    if let Some(mark) = &edge.mark {
                        out.narrowed.push(Narrowed {
                            by: node.id().to_string(),
                            mark: mark.clone(),
                        });
                        any = true;
                    }
                }
                _ => {}
            }
        }
    }
    any.then_some(out)
}

/// D2: `text` with each narrowed passage wrapped where it stands.
pub fn retracted(text: &str, retraction: &Retraction) -> Option<String> {
    let mut marks: Vec<&Narrowed> = retraction
        .narrowed
        .iter()
        .filter(|narrow| !narrow.mark.is_empty())
        .collect();
    if marks.is_empty() {
        return None;
    }
    marks.sort_by_key(|narrow| std::cmp::Reverse(narrow.mark.len()));
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    'scan: while !rest.is_empty() {
        for narrow in &marks {
            if rest.starts_with(&narrow.mark) {
                out.push_str(&format!("[[retracted by {}: {}]]", narrow.by, narrow.mark));
                rest = &rest[narrow.mark.len()..];
                continue 'scan;
            }
        }
        let ch = rest.chars().next().expect("rest is not empty");
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    (out != text).then_some(out)
}

/// V5: handover's warning counts, in the order handover reports them.
pub fn warnings(all: &[Node]) -> Vec<Warning> {
    let in_progress: Vec<&Node> = all
        .iter()
        .filter(|node| {
            matches!(
                node.state(),
                Some(RequirementState::Filed | RequirementState::Approved)
            )
        })
        .collect();
    let superseded = node_ids(all, Relation::Supersedes);
    let unrecorded: BTreeSet<String> = all
        .iter()
        .filter(|node| unrecorded_scope(node))
        .map(|node| node.id().to_string())
        .collect();
    let mut out = Vec::new();
    push(
        &mut out,
        "in-progress requirements relying on a superseded decision",
        relying_on(&in_progress, &superseded),
    );
    push(
        &mut out,
        "in-progress requirements relying on an unrecorded decision scope",
        relying_on(&in_progress, &unrecorded),
    );
    push(
        &mut out,
        "criteria with an empty body",
        all.iter()
            .filter(|node| node.kind() == NodeKind::Criterion && node.body().trim().is_empty())
            .count(),
    );
    push(
        &mut out,
        "open questions nobody waits on",
        all.iter().filter(|node| unwaited(node, all)).count(),
    );
    push(
        &mut out,
        "criteria unmet with every need closed",
        all.iter()
            .filter(|node| node.kind() == NodeKind::Criterion && orphaned(node, all))
            .count(),
    );
    out
}

fn node_ids(all: &[Node], relation: Relation) -> BTreeSet<String> {
    all.iter()
        .flat_map(|node| edges(node, relation))
        .map(|id| id.to_string())
        .collect()
}

fn unrecorded_scope(node: &Node) -> bool {
    matches!(node.data(), NodeData::Decision(data) if data.scope.is_unrecorded())
}

fn relying_on(in_progress: &[&Node], targets: &BTreeSet<String>) -> usize {
    in_progress
        .iter()
        .filter(|node| {
            relies_on(node)
                .iter()
                .any(|id| targets.contains(&id.to_string()))
        })
        .count()
}

fn push(out: &mut Vec<Warning>, label: &str, count: usize) {
    if count > 0 {
        out.push(Warning {
            label: label.to_string(),
            count,
        });
    }
}
