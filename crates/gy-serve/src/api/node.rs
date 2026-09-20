//! GET /api/node/<id or alias>: the `show` view of one node, with each edge's
//! peer named and the node's own writes (n-bd52). No judgement here.
use crate::http::Response;
use gy_ledger::{
    EdgeLine, Ego, Filter, Listing, LogRow, NodeData, NodeKind, Repository, Retraction, Store, ego,
    list, show,
};
use serde::Serialize;
use std::collections::BTreeMap;

/// One edge with its peer's title, kind, and first alias; a peer that is gone
/// keeps only the id.
#[derive(Serialize)]
struct Edge {
    name: String,
    to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<NodeKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
}

/// The show view, with the named edges and the node's own writes.
#[derive(Serialize)]
struct Node {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    kind: NodeKind,
    title: String,
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body_marked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision_scope_marked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    aliases: Vec<String>,
    data: NodeData,
    #[serde(skip_serializing_if = "Option::is_none")]
    cancellation: Option<Retraction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    attributes: BTreeMap<String, String>,
    edges: Vec<Edge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    neighborhood: Option<Ego>,
    history: Vec<LogRow>,
}

pub fn node<S: Store>(repo: &Repository<S>, id: &str) -> Response {
    let shown = match show(repo, &[id.to_string()], true) {
        Ok(mut shown) if !shown.is_empty() => shown.remove(0),
        Ok(_) => return Response::text(404, "not found"),
        Err(_) => return Response::text(404, "not found"),
    };
    let edges = shown.edges.iter().map(|edge| named(repo, edge)).collect();
    let neighborhood = ego(repo, &shown.id, 2).ok();
    let history = history(repo, &shown.id);
    Response::json(&Node {
        id: shown.id,
        reference: shown.reference,
        kind: shown.kind,
        title: shown.title,
        body: shown.body,
        body_marked: shown.body_marked,
        decision_scope_marked: shown.decision_scope_marked,
        scope: shown.scope,
        created: shown.created,
        aliases: shown.aliases,
        data: shown.data,
        cancellation: shown.cancellation,
        missing: shown.missing,
        attributes: shown.attributes,
        edges,
        neighborhood,
        history,
    })
}

/// One edge and the peer it points at.
fn named<S: Store>(repo: &Repository<S>, edge: &EdgeLine) -> Edge {
    let peer = repo
        .resolve(&edge.to)
        .ok()
        .and_then(|id| repo.get(&id).ok().flatten());
    Edge {
        name: edge.name.clone(),
        to: edge.to.clone(),
        mark: edge.mark.clone(),
        title: peer.as_ref().map(|node| node.title().to_string()),
        kind: peer.as_ref().map(|node| node.kind()),
        alias: peer
            .as_ref()
            .and_then(|node| node.aliases().first().map(|alias| alias.0.clone())),
    }
}

/// This node's writes, newest first.
fn history<S: Store>(repo: &Repository<S>, id: &str) -> Vec<LogRow> {
    let filter = Filter {
        since: Some(0),
        ..Default::default()
    };
    match list(repo, &filter) {
        Ok(Listing::History(rows)) => rows
            .into_iter()
            .filter(|row| row.node == id)
            .rev()
            .collect(),
        _ => Vec::new(),
    }
}
