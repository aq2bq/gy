//! The routes: plain functions over `http`, testable without a socket (ac-a49a).
pub mod graph;
pub mod history;
pub mod labels;
pub mod list;
pub mod node;
pub mod now;
pub mod search;
pub mod shell;
pub mod ticks;
pub mod wait;

use crate::assets;
use crate::http::{Request, Response};
use gy_ledger::{Repository, Store};

/// The one entry point: any other method is 405, any other path is 404.
pub fn route<S: Store>(repo: &Repository<S>, req: &Request, name: &str) -> Response {
    if req.method != "GET" {
        return Response::method_not_allowed();
    }
    match req.path.as_str() {
        "/" | "/index.html" => index(req, name),
        "/api/shell" => shell::shell(repo, req),
        "/api/now" => now::answer(repo, req),
        "/api/history" => history::answer(repo, req),
        "/api/labels" => labels::labels(repo, req),
        "/api/list" => list::rows(repo, req),
        "/api/search" => search::search(repo, req),
        "/api/ticks" => ticks::ticks(repo, req),
        _ => match req.path.strip_prefix("/api/node/") {
            Some(id) => node::node(repo, &crate::http::decode(id, false)),
            None => static_file(req),
        },
    }
}

/// The first HTML; the language comes from `Accept-Language` (ac-b9b1) and the
/// tab's title from the caller (n-07f0).
fn index(req: &Request, name: &str) -> Response {
    let ja = req
        .header("accept-language")
        .is_some_and(|value| value.trim_start().starts_with("ja"));
    let title = if name.is_empty() {
        "gy".to_string()
    } else {
        format!("gy - {}", escape_html(name))
    };
    Response::html(
        assets::index()
            /* The fixed word first, the name last: a name cannot be caught by
            a later replacement (a directory named `%LANG%` stays itself). */
            .replace("%LANG%", if ja { "ja" } else { "en" })
            .replace("%NAME%", &title),
    )
}

/// What a name may not carry into the `<title>`: the four characters that could
/// close it or open a tag. `&` goes first, so an escaped name is not escaped
/// twice.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
