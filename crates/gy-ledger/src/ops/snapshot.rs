//! One pass over the store for a caller that reads many nodes (n-2e03). The
//! nodes in store order, an id index, and the reverse edges; also the one
//! resolution judgement, shared with `Repository`.
use crate::model::{Edge, Node, NodeData, NodeId};
use crate::store::{Error, Result};
use std::collections::BTreeMap;

/// Everything a caller that reads many nodes needs, from one store pass.
pub(crate) struct Snapshot {
    nodes: Vec<Node>,
    by_id: BTreeMap<String, usize>,
    incoming: BTreeMap<String, Vec<Edge>>,
}
impl Snapshot {
    pub(crate) fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    pub(crate) fn get(&self, id: &NodeId) -> Option<&Node> {
        self.by_id
            .get(&id.to_string().to_lowercase())
            .map(|index| &self.nodes[*index])
    }
    /// An exact id first, so a caller that names ids stays linear; an alias or
    /// a reference falls back to the one scan resolve uses.
    pub(crate) fn resolve(&self, text: &str) -> Result<NodeId> {
        if let Some(index) = self.by_id.get(&text.to_lowercase()) {
            return Ok(self.nodes[*index].id().clone());
        }
        resolve_in(&self.nodes, text)
    }
    pub(crate) fn incoming(&self, id: &NodeId) -> &[Edge] {
        self.incoming
            .get(&id.to_string())
            .map_or(&[], Vec::as_slice)
    }
}

/// Build the index from already-loaded nodes, in their order.
pub(crate) fn build(nodes: Vec<Node>) -> Snapshot {
    let mut by_id = BTreeMap::new();
    let mut incoming: BTreeMap<String, Vec<Edge>> = BTreeMap::new();
    for (index, node) in nodes.iter().enumerate() {
        by_id.insert(node.id().to_string().to_lowercase(), index);
        for edge in node.links() {
            incoming.entry(edge.to.to_string()).or_default().push(Edge {
                from: edge.to.clone(),
                label: edge.label,
                reversed: true,
                mark: edge.mark.clone(),
                to: edge.from.clone(),
            });
        }
    }
    Snapshot {
        nodes,
        by_id,
        incoming,
    }
}

/// Resolve an id, alias, or requirement ref against already-loaded nodes. One
/// judgement, shared by `Repository::resolve` and `Snapshot::resolve`.
pub(crate) fn resolve_in(all: &[Node], text: &str) -> Result<NodeId> {
    let wanted = text.to_lowercase();
    let old = normalize(text);
    let (mut by_id, mut by_alias, mut by_ref) = (Vec::new(), Vec::new(), Vec::new());
    for node in all {
        let key = node.id().to_string();
        if key.to_lowercase() == wanted {
            by_id.push(node.id().clone());
        } else if node
            .aliases()
            .iter()
            .any(|alias| normalize(&alias.0) == old)
        {
            by_alias.push(node.id().clone());
        } else if referenced(node).is_some_and(|ref_| ref_ == text || ref_.ends_with(text)) {
            by_ref.push(node.id().clone());
        }
    }
    if by_id.is_empty() {
        choose(
            text,
            if by_alias.is_empty() {
                by_ref
            } else {
                by_alias
            },
        )
    } else {
        choose(text, by_id)
    }
}

/// The reverse of every edge that points at `id`, over already-loaded nodes.
pub(crate) fn incoming_in(all: &[Node], id: &NodeId) -> Vec<Edge> {
    let mut edges = Vec::new();
    for node in all {
        for edge in node.links() {
            if &edge.to == id {
                edges.push(Edge {
                    from: id.clone(),
                    label: edge.label,
                    reversed: true,
                    mark: edge.mark.clone(),
                    to: edge.from.clone(),
                });
            }
        }
    }
    edges
}

fn names(matches: &[NodeId]) -> String {
    matches
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A requirement's outward reference, if any.
fn referenced(node: &Node) -> Option<&str> {
    match node.data() {
        NodeData::Requirement(requirement) => requirement.reference.as_ref().map(|r| r.0.as_str()),
        _ => None,
    }
}

fn choose(text: &str, mut matches: Vec<NodeId>) -> Result<NodeId> {
    match matches.len() {
        0 => Err(Error::invalid(format!("no node matches {text}"))),
        1 => Ok(matches.remove(0)),
        _ => Err(Error::invalid(format!(
            "{text} matches several nodes: {}",
            names(&matches)
        ))),
    }
}

/// Lowercase the kind and drop leading zeros from an all-digit suffix, so
/// `D-06` and `d-6` name the same old id.
fn normalize(text: &str) -> String {
    match text.rsplit_once('-') {
        Some((kind, suffix)) if suffix.bytes().all(|byte| byte.is_ascii_digit()) => {
            let digits = suffix.trim_start_matches('0');
            format!(
                "{}-{}",
                kind.to_lowercase(),
                if digits.is_empty() { "0" } else { digits }
            )
        }
        _ => text.to_lowercase(),
    }
}
