//! GET /api/history: the list view's history mode as JSON (n-4a08). The rows
//! and their order come from gy-ledger; this file filters by scope and shapes
//! the answer. It does not judge a state or a decider.
use crate::http::{Request, Response};
use gy_ledger::{Filter, Listing, LogRow, Repository, Result, Store, list};
use serde::Serialize;
use std::collections::BTreeMap;

/// The most rows one answer carries; older writes are left out (n-4a08).
const LIMIT: usize = 200;

#[derive(Serialize)]
struct Answer<'a> {
    total: usize,
    actors: Vec<String>,
    rows: &'a [LogRow],
}

pub fn answer<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    let since = match seq_param(req) {
        Ok(seq) => seq,
        Err(response) => return response,
    };
    let filter = Filter {
        actor: req.param("actor").map(ToString::to_string),
        since: Some(since),
        ..Default::default()
    };
    let rows = match list(repo, &filter) {
        Ok(Listing::History(rows)) => rows,
        Ok(Listing::Nodes(_)) => Vec::new(),
        Err(error) => return Response::text(400, &error.to_string()),
    };
    let (actors, scopes) = match (writers(repo), scopes(repo)) {
        (Ok(actors), Ok(scopes)) => (actors, scopes),
        (Err(error), _) | (_, Err(error)) => return Response::text(500, &error.to_string()),
    };
    let mut rows: Vec<LogRow> = rows
        .into_iter()
        .filter(|row| {
            in_scope(
                req.param("scope"),
                scopes.get(&row.node).map(String::as_str),
            )
        })
        .collect();
    rows.reverse();
    let total = rows.len();
    let limit = limit_param(req);
    Response::json(&Answer {
        total,
        actors,
        rows: &rows[..total.min(limit)],
    })
}

/// The `limit` count: 1..=LIMIT, and anything else is the default. The cap
/// only trims the rows, never `total` (n-b963).
fn limit_param(req: &Request) -> usize {
    req.param("limit")
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|count| (1..=LIMIT).contains(count))
        .unwrap_or(LIMIT)
}

/// The `since` sequence, 0 when absent; a broken value is 400.
fn seq_param(req: &Request) -> std::result::Result<u64, Response> {
    match req.param("since") {
        Some(text) => text
            .parse()
            .map_err(|_| Response::text(400, &format!("bad since {text}"))),
        None => Ok(0),
    }
}

/// Every writer as a reader sees them, in name order, from the whole
/// history (n-4a08, n-d36d).
fn writers<S: Store>(repo: &Repository<S>) -> Result<Vec<String>> {
    let rows = match list(
        repo,
        &Filter {
            since: Some(0),
            ..Default::default()
        },
    )? {
        Listing::History(rows) => rows,
        Listing::Nodes(_) => Vec::new(),
    };
    let mut actors: Vec<String> = rows.into_iter().map(|row| row.who).collect();
    actors.sort();
    actors.dedup();
    Ok(actors)
}

/// The scope of every node, so a history row can be filtered by it.
fn scopes<S: Store>(repo: &Repository<S>) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    for node in repo.all()? {
        map.insert(node.id().to_string(), node.scope().to_string());
    }
    Ok(map)
}

/// The scope filter: no scope, or `all`, keeps every row.
fn in_scope(scope: Option<&str>, row_scope: Option<&str>) -> bool {
    scope.is_none_or(|scope| scope == "all" || row_scope == Some(scope))
}
