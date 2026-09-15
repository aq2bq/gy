//! GET /api/graph: the layout of the asked point (n-796b, d-b93b). The answer
//! carries ids, kinds, scopes, degrees, edges, and coordinates, never titles.
use crate::graph_cache::GraphCache;
use crate::http::{Request, Response};
use crate::layout::{self};
use crate::server::{Opened, Opener};
use gy_ledger::{Filter, Listing, Node, Repository, Store, list};
use std::collections::HashMap;
use std::sync::Arc;

/// `at` picks the point; `scope` narrows it.
pub fn answer(open: &Opener, cache: &GraphCache, at: Option<u64>, req: &Request) -> Response {
    let scope = req
        .param("scope")
        .filter(|text| *text != "all")
        .map(ToString::to_string);
    match open(at) {
        Ok(Opened::Now(repo)) => graph(&repo, cache, scope.as_deref()),
        Ok(Opened::At(repo)) => graph(&repo, cache, scope.as_deref()),
        Err(error) => Response::text(500, &error.to_string()),
    }
}

/// The layout of one ledger, from the cache when it is still here.
pub fn graph<S: Store>(repo: &Repository<S>, cache: &GraphCache, scope: Option<&str>) -> Response {
    let seq = repo.store().history().last().map_or(0, |entry| entry.seq);
    let key = (seq, scope.map(ToString::to_string));
    if let Some(graph) = cache.get(&key) {
        return Response::json(&*graph);
    }
    let nodes: Vec<Node> = repo
        .all()
        .unwrap_or_default()
        .into_iter()
        .filter(|node| scope.is_none_or(|scope| node.scope() == scope))
        .collect();
    let graph = Arc::new(layout::layout(&nodes, &states(repo), seq));
    cache.put(key, Arc::clone(&graph));
    Response::json(&*graph)
}

/// Every node's state word, straight from the list view, so the picture shows
/// what is still open without judging it here (a decision has no word).
fn states<S: Store>(repo: &Repository<S>) -> HashMap<String, String> {
    match list(repo, &Filter::default()) {
        Ok(Listing::Nodes(rows)) => rows
            .into_iter()
            .filter_map(|row| row.status.map(|state| (row.id, state)))
            .collect(),
        _ => HashMap::new(),
    }
}
