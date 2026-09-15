//! GET /api/search?q=<text>: the rows whose id, alias, or title contains the
//! text, at most ten, exact ids and aliases first (n-bd52).
use crate::http::{Request, Response};
use gy_ledger::{Filter, Listing, NodeKind, Repository, Row, Store, list};
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
    let query = req.param("q").unwrap_or_default().trim().to_lowercase();
    if query.is_empty() {
        return Response::json(&Hits { hits: Vec::new() });
    }
    let rows = match list(repo, &Filter::default()) {
        Ok(Listing::Nodes(rows)) => rows,
        _ => Vec::new(),
    };
    let aliases = aliases(repo);
    let mut exact = Vec::new();
    let mut rest = Vec::new();
    for row in rows {
        let alias = aliases.get(&row.id).cloned();
        if !matches(&query, &row, alias.as_deref()) {
            continue;
        }
        let hit = Hit {
            id: row.id,
            alias,
            kind: row.kind,
            title: row.title,
            scope: row.scope,
            status: row.status,
        };
        if exact_id_or_alias(&query, &hit) {
            exact.push(hit);
        } else {
            rest.push(hit);
        }
    }
    exact.extend(rest);
    Response::json(&Hits {
        hits: exact.into_iter().take(LIMIT).collect(),
    })
}

/// Whether the row matches on its id, its alias, or its title.
fn matches(query: &str, row: &Row, alias: Option<&str>) -> bool {
    row.id.to_lowercase().contains(query)
        || row.title.to_lowercase().contains(query)
        || alias.is_some_and(|alias| alias.to_lowercase().contains(query))
}

/// Whether the id or the alias is the query itself, so it sorts first.
fn exact_id_or_alias(query: &str, hit: &Hit) -> bool {
    hit.id.to_lowercase() == query
        || hit
            .alias
            .as_deref()
            .is_some_and(|alias| alias.to_lowercase() == query)
}

/// Every node's first alias, by id. One pass instead of one `resolve` per row.
fn aliases<S: Store>(repo: &Repository<S>) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for node in repo.all().unwrap_or_default() {
        if let Some(alias) = node.aliases().first() {
            out.insert(node.id().to_string(), alias.0.clone());
        }
    }
    out
}
