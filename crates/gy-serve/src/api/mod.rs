//! The routes: plain functions over `http`, testable without a socket (ac-a49a).
pub mod list;
pub mod node;
pub mod now;
pub mod search;
pub mod shell;

use crate::assets;
use crate::http::{Request, Response};
use gy_ledger::{Repository, Store};

/// The one entry point: any other method is 405, any other path is 404.
pub fn route<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    if req.method != "GET" {
        return Response::method_not_allowed();
    }
    match req.path.as_str() {
        "/" | "/index.html" => index(req),
        "/api/shell" => shell::shell(repo, req),
        "/api/now" => now::answer(repo, req),
        "/api/list" => list::rows(repo, req),
        "/api/search" => search::search(repo, req),
        _ => match req.path.strip_prefix("/api/node/") {
            Some(id) => node::node(repo, &crate::http::decode(id, false)),
            None => static_file(req),
        },
    }
}

/// The first HTML; the language comes from `Accept-Language` (ac-b9b1).
fn index(req: &Request) -> Response {
    let ja = req
        .header("accept-language")
        .is_some_and(|value| value.trim_start().starts_with("ja"));
    Response::html(assets::index().replace("%LANG%", if ja { "ja" } else { "en" }))
}

/// `/assets/<name>`, or 404: a missing file and a `..` path are the same.
fn static_file(req: &Request) -> Response {
    match req.path.strip_prefix("/assets/").and_then(assets::get) {
        Some((body, content_type)) => Response {
            status: 200,
            content_type,
            body: body.as_bytes().to_vec(),
        },
        None => Response::not_found(),
    }
}
