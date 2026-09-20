//! ego: the focus with everything within a few hops, for the node page's
//! connection map (n-...). The whole graph is read once, the nearness comes
//! from a breadth-first walk over the undirected edges, and the cap keeps the
//! nearest, so the card never draws an unbounded picture.
use crate::model::{Edge, Node, NodeKind};
use crate::ops::repository::{Repository, Result, Store};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// The cap on the nodes one neighbourhood carries, the focus included.
pub const EGO_LIMIT: usize = 40;

/// One node of the neighbourhood: how far it stands from the focus.
#[derive(Debug, Clone, Serialize)]
pub struct EgoNode {
    pub id: String,
    pub hop: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<NodeKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// One edge between two nodes of the neighbourhood, named from each side: the
/// `from` side reads `name`, the `to` side reads `inverse`.
#[derive(Debug, Clone, Serialize)]
pub struct EgoEdge {
    pub from: String,
    pub to: String,
    pub name: String,
    pub inverse: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<String>,
}

/// The focus and its neighbourhood: its id, the nodes (nearest first), the
/// edges between them, and how many the cap left out.
#[derive(Debug, Clone, Serialize)]
pub struct Ego {
    pub root: String,
    pub nodes: Vec<EgoNode>,
    pub edges: Vec<EgoEdge>,
    pub truncated: usize,
}

/// The focus and everything within `depth` hops. An edge to a node that is gone
/// still counts for nearness, and such a node is kept with only its id.
pub fn ego<S: Store>(repo: &Repository<S>, text: &str, depth: usize) -> Result<Ego> {
    let root = repo.resolve(text)?.to_string();
    let all = repo.all()?;
    let (order, hop) = walk(&root, &adjacency(&all), depth);
    let truncated = order.len().saturating_sub(EGO_LIMIT);
    let kept: BTreeSet<&str> = order.iter().take(EGO_LIMIT).map(String::as_str).collect();
    Ok(Ego {
        root,
        nodes: nodes(&all, &order, &hop),
        edges: edges(&all, &kept),
        truncated,
    })
}

/// Every edge, read from both ends in one pass over the nodes (D-76: the edge
/// lives on its `from` side only, so the nearness is assembled here).
fn adjacency(nodes: &[Node]) -> BTreeMap<String, BTreeSet<String>> {
    let mut near: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for node in nodes {
        let from = node.id().to_string();
        for edge in node.links() {
            let to = edge.to.to_string();
            near.entry(from.clone()).or_default().insert(to.clone());
            near.entry(to).or_default().insert(from.clone());
        }
    }
    near
}

/// The kept ids, nearest first, each with its peer named.
fn nodes(all: &[Node], order: &[String], hop: &BTreeMap<String, usize>) -> Vec<EgoNode> {
    let by_id: BTreeMap<String, &Node> = all
        .iter()
        .map(|node| (node.id().to_string(), node))
        .collect();
    order
        .iter()
        .take(EGO_LIMIT)
        .map(|id| {
            let node = by_id.get(id).copied();
            EgoNode {
                id: id.clone(),
                hop: hop[id],
                kind: node.map(Node::kind),
                alias: node.and_then(|node| node.aliases().first().map(|alias| alias.0.clone())),
                title: node.map(|node| node.title().to_string()),
            }
        })
        .collect()
}

/// The edges whose both ends are kept, named from each side.
fn edges(all: &[Node], kept: &BTreeSet<&str>) -> Vec<EgoEdge> {
    let mut edges = Vec::new();
    for node in all {
        let from = node.id().to_string();
        if !kept.contains(from.as_str()) {
            continue;
        }
        for edge in node.links() {
            let to = edge.to.to_string();
            if !kept.contains(to.as_str()) {
                continue;
            }
            edges.push(EgoEdge {
                from: from.clone(),
                to,
                name: edge.name().to_string(),
                inverse: other_name(edge),
                mark: edge.mark.clone(),
            });
        }
    }
    edges
}

/// The relation's name as the edge's `to` side reads it.
fn other_name(edge: &Edge) -> String {
    if edge.reversed {
        edge.label.name()
    } else {
        edge.label.inverse()
    }
    .to_string()
}

/// A breadth-first walk over the undirected edges: the ids in the order they
/// were first reached, and how far each one stands.
fn walk(
    root: &str,
    near: &BTreeMap<String, BTreeSet<String>>,
    depth: usize,
) -> (Vec<String>, BTreeMap<String, usize>) {
    let mut order = vec![root.to_string()];
    let mut hop = BTreeMap::from([(root.to_string(), 0)]);
    let mut queue = VecDeque::from([root.to_string()]);
    while let Some(at) = queue.pop_front() {
        let step = hop[&at];
        if step == depth {
            continue;
        }
        for next in near.get(&at).into_iter().flatten() {
            if hop.contains_key(next) {
                continue;
            }
            hop.insert(next.clone(), step + 1);
            order.push(next.clone());
            queue.push_back(next.clone());
        }
    }
    (order, hop)
}
