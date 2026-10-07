//! GET /api/search?q=<text>: the rows whose id, alias, title, or body contains
//! the text, at most ten, exact ids and aliases first, then titles, then bodies
//! (n-bd52, d-6a45).
use crate::http::{Request, Response};
use gy_ledger::{Filter, Listing, Node, NodeKind, Repository, Store, list, matches};
use serde::Serialize;
use std::collections::HashMap;

/// How many hits search answers with.
const LIMIT: usize = 10;

/// One hit: the row's id, alias, kind, title, scope, and state word.
#[derive(Serialize)]
struct Hit {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
    kind: NodeKind,
    title: String,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

#[derive(Serialize)]
struct Hits {
    hits: Vec<Hit>,
}

pub fn search<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    let query = req.param("q").unwrap_or_default().trim().to_string();
    if query.is_empty() {
        return Response::json(&Hits { hits: Vec::new() });
    }
    let rows = match list(repo, &Filter::default()) {
        Ok(Listing::Nodes(rows)) => rows,
        _ => Vec::new(),
    };
    let nodes: HashMap<String, Node> = repo
        .all()
        .unwrap_or_default()
        .into_iter()
        .map(|node| (node.id().to_string(), node))
        .collect();
    let mut hits = Vec::new();
    for row in rows {
        let Some(node) = nodes.get(&row.id) else {
            continue;
        };
        let Some(grade) = matches(node, &query) else {
            continue;
        };
        let alias = node.aliases().first().map(|alias| alias.0.clone());
        hits.push((
            grade,
            Hit {
                id: row.id,
                alias,
                kind: row.kind,
                title: row.title,
                scope: row.scope,
                status: row.status,
            },
        ));
    }
    hits.sort_by_key(|(grade, _)| *grade);
    Response::json(&Hits {
        hits: hits.into_iter().take(LIMIT).map(|(_, hit)| hit).collect(),
    })
}
