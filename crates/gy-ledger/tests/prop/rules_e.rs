//! The E5–E8, R5, C1 and C2 invariant checks (n-a006): against a pair of
//! snapshots for an accepted step. `prop_p1b` reads this with `#[path]`, so
//! the E1–E4 test crate does not carry them.
use crate::prop::State;
use crate::prop::rules::{find, fwd, is_approved, is_closed_need};
use gy_ledger::{Closure, Edge, Node, NodeData, NodeId, NodeKind, Relation, RequirementState};
use std::collections::{BTreeMap, BTreeSet};

/// The targets of a node's edges with one relation.
pub fn edges(node: &Node, relation: Relation) -> BTreeSet<String> {
    fwd(node)
        .filter(|edge| edge.label == relation)
        .map(|edge| edge.to.to_string())
        .collect()
}

/// A requirement whose state is done (C3): done and approved both cover.
pub fn is_done_requirement(node: &Node) -> bool {
    node.state() == Some(RequirementState::Done)
}

pub fn is_satisfied(node: &Node) -> bool {
    matches!(node.data(), NodeData::Criterion(data) if data.satisfied)
}

fn evidence(node: &Node) -> Option<String> {
    match node.data() {
        NodeData::Criterion(data) => data.evidence.clone(),
        _ => None,
    }
}

fn closure_is_decision(node: &Node) -> bool {
    matches!(node.data(), NodeData::Question(data) if data.closure == Some(Closure::Decision))
}

fn key(edge: &Edge) -> (String, &'static str, String) {
    (
        edge.from.to_string(),
        edge.label.name(),
        edge.to.to_string(),
    )
}

/// R5 freezes an approved requirement, and a criterion an approved requirement
/// targets.
pub fn node_frozen(nodes: &[Node], id: &NodeId) -> bool {
    match find(nodes, id) {
        Some(node) if is_approved(node) => true,
        _ => nodes.iter().any(|req| {
            is_approved(req)
                && fwd(req).any(|edge| edge.label == Relation::Targets && &edge.to == id)
        }),
    }
}

/// C3: a criterion is covered iff an approved or done requirement targets it.
pub fn covered(nodes: &[Node], criterion: &NodeId) -> bool {
    nodes.iter().any(|req| {
        (is_approved(req) || is_done_requirement(req))
            && fwd(req).any(|edge| edge.label == Relation::Targets && &edge.to == criterion)
    })
}

/// E5: the mark of an added narrows / supersedes is found in the older
/// decision's body or scope note.
pub fn mark_found(nodes: &[Node], to: &NodeId, mark: Option<&str>) -> bool {
    let Some(mark) = mark else {
        return true;
    };
    find(nodes, to).is_some_and(|node| {
        node.body().contains(mark)
            || matches!(node.data(), NodeData::Decision(data) if data.scope.text().contains(mark))
    })
}

/// E5: an added narrows / supersedes carries a mark that resolves.
pub fn broken_e5(before: &State, after: &State) -> bool {
    let lineage = |edge: &Edge| matches!(edge.label, Relation::Narrows | Relation::Supersedes);
    let was: BTreeSet<_> = before
        .nodes
        .iter()
        .flat_map(fwd)
        .filter(|edge| lineage(edge))
        .map(key)
        .collect();
    after
        .nodes
        .iter()
        .flat_map(fwd)
        .filter(|edge| lineage(edge))
        .any(|edge| {
            !was.contains(&key(edge)) && !mark_found(&after.nodes, &edge.to, edge.mark.as_deref())
        })
}

/// E6: an added closes comes from a question closed by decision.
pub fn broken_e6(before: &State, after: &State) -> bool {
    let was: BTreeSet<_> = before
        .nodes
        .iter()
        .flat_map(fwd)
        .filter(|edge| edge.label == Relation::Closes)
        .map(key)
        .collect();
    after
        .nodes
        .iter()
        .flat_map(fwd)
        .filter(|edge| edge.label == Relation::Closes)
        .any(|edge| {
            !was.contains(&key(edge))
                && !find(&after.nodes, &edge.from).is_some_and(closure_is_decision)
        })
}

/// E7: both graphs stay acyclic.
pub fn broken_e7(_before: &State, after: &State) -> bool {
    has_cycle(&after.nodes, &[Relation::DependsOn])
        || has_cycle(
            &after.nodes,
            &[
                Relation::Narrows,
                Relation::Widens,
                Relation::Supersedes,
                Relation::Completes,
            ],
        )
}

/// E8: a need that gained a filed-as is not closed.
pub fn broken_e8(before: &State, after: &State) -> bool {
    after
        .nodes
        .iter()
        .filter(|node| node.kind() == NodeKind::Need)
        .any(|node| {
            let was = find(&before.nodes, node.id())
                .map(|old| edges(old, Relation::FiledAs))
                .unwrap_or_default();
            edges(node, Relation::FiledAs)
                .difference(&was)
                .next()
                .is_some()
                && is_closed_need(node)
        })
}

/// R5: while a requirement stays approved, its frozen fields and the title and
/// body of a criterion it targets do not change.
pub fn broken_r5(before: &State, after: &State) -> bool {
    for req in &after.nodes {
        if !is_approved(req) {
            continue;
        }
        let Some(was) = find(&before.nodes, req.id()) else {
            continue;
        };
        if !is_approved(was) {
            continue;
        }
        if was.title() != req.title()
            || was.body() != req.body()
            || edges(was, Relation::Targets) != edges(req, Relation::Targets)
            || edges(was, Relation::ReliesOn) != edges(req, Relation::ReliesOn)
        {
            return true;
        }
    }
    for criterion in &after.nodes {
        if criterion.kind() != NodeKind::Criterion {
            continue;
        }
        let Some(was) = find(&before.nodes, criterion.id()) else {
            continue;
        };
        if was.title() == criterion.title() && was.body() == criterion.body() {
            continue;
        }
        if node_frozen(&after.nodes, criterion.id()) {
            return true;
        }
    }
    false
}

/// C1: a criterion turned satisfied only where it was covered before.
pub fn broken_c1(before: &State, after: &State) -> bool {
    after
        .nodes
        .iter()
        .filter(|node| is_satisfied(node))
        .any(|node| {
            let was = find(&before.nodes, node.id()).is_some_and(is_satisfied);
            !was && !covered(&before.nodes, node.id())
        })
}

/// C2: a satisfied criterion has evidence and was not satisfied before.
pub fn broken_c2(before: &State, after: &State) -> bool {
    after
        .nodes
        .iter()
        .filter(|node| is_satisfied(node))
        .any(|node| {
            let now = evidence(node);
            if now.as_deref().is_none_or(|text| text.trim().is_empty()) {
                return true;
            }
            find(&before.nodes, node.id())
                .is_some_and(|was| is_satisfied(was) && evidence(was) != now)
        })
}

fn has_cycle(nodes: &[Node], labels: &[Relation]) -> bool {
    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in nodes {
        for edge in fwd(node) {
            if labels.contains(&edge.label) {
                graph
                    .entry(edge.from.to_string())
                    .or_default()
                    .push(edge.to.to_string());
            }
        }
    }
    fn reaches(
        from: &str,
        graph: &BTreeMap<String, Vec<String>>,
        state: &mut BTreeMap<String, u8>,
    ) -> bool {
        state.insert(from.to_string(), 1);
        if let Some(next) = graph.get(from) {
            for id in next {
                match state.get(id).copied().unwrap_or(0) {
                    1 => return true,
                    0 if reaches(id, graph, state) => return true,
                    _ => {}
                }
            }
        }
        state.insert(from.to_string(), 2);
        false
    }
    let mut state: BTreeMap<String, u8> = BTreeMap::new();
    let ids: Vec<String> = graph.keys().cloned().collect();
    ids.iter()
        .any(|id| state.get(id).copied().unwrap_or(0) == 0 && reaches(id, &graph, &mut state))
}
