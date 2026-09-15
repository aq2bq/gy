//! GET /api/list: the `list` view's rows as JSON (n-bd52). The rows, their
//! order, and their words come from gy-ledger; this file only filters.
use crate::http::{Request, Response};
use gy_ledger::{Filter, Listing, NodeKind, Repository, Row, Store, list};
use serde::Serialize;

/// The rows the view returned, in its order.
#[derive(Serialize)]
struct Rows {
    rows: Vec<Row>,
}

pub fn rows<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    let kind = match req.param("kind") {
        Some(text) => match parse_kind(text) {
            Some(kind) => Some(kind),
            None => return Response::text(400, &format!("unknown kind {text}")),
        },
        None => None,
    };
    let filter = Filter {
        kind,
        status: req.param("status").map(ToString::to_string),
        grep: req.param("q").map(ToString::to_string),
        ..Default::default()
    };
    let rows = match list(repo, &filter) {
        Ok(Listing::Nodes(rows)) => rows,
        Ok(Listing::History(_)) => Vec::new(),
        Err(error) => return Response::text(400, &error.to_string()),
    };
    let scope = req.param("scope");
    let rows = rows
        .into_iter()
        .filter(|row| in_scope(scope, &row.scope))
        .collect();
    Response::json(&Rows { rows })
}

/// The scope filter: no scope, or `all`, keeps every row.
fn in_scope(scope: Option<&str>, row_scope: &str) -> bool {
    scope.is_none_or(|scope| scope == "all" || scope == row_scope)
}

/// The kind by its name (`need` or `Need`) or its prefix (`n`), as the CLI
/// reads it.
fn parse_kind(text: &str) -> Option<NodeKind> {
    let wanted = text.to_lowercase();
    NodeKind::ALL
        .into_iter()
        .find(|kind| kind.name() == wanted || kind.prefix() == wanted)
}
