//! GET /api/list: the `list` view's rows as JSON (n-bd52). The rows, their
//! order, and their words come from gy-ledger; this file shapes the answer.
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
    let mut filter = Filter::default();
    filter.scope = req
        .param("scope")
        .filter(|scope| *scope != "all")
        .map(str::to_string);
    filter.kind = kind;
    filter.status = req.param("status").map(ToString::to_string);
    filter.grep = req.param("q").map(ToString::to_string);
    let rows = match list(repo, &filter) {
        Ok(Listing::Nodes(rows)) => rows,
        Ok(Listing::History(_)) => Vec::new(),
        Err(error) => return Response::text(400, &error.to_string()),
    };
    Response::json(&Rows { rows })
}

/// The kind by its name (`need` or `Need`) or its prefix (`n`), as the CLI
/// reads it.
fn parse_kind(text: &str) -> Option<NodeKind> {
    let wanted = text.to_lowercase();
    NodeKind::ALL
        .into_iter()
        .find(|kind| kind.name() == wanted || kind.prefix() == wanted)
}
