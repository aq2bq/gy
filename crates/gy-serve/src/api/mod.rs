//! The routes: plain functions over `http`, testable without a socket (ac-a49a).
//!
//! Three of them are the reading surface the three screens use (d-6d90):
//! `overview` (the first screen), `cross` (the crossing screen), and `path`
//! (the detail screen). Each file's doc comment is the shape of its JSON and
//! the meaning of every key. The pages that came before the three keep their
//! own answers too (`now`, `list`, `history`, and `labels`), because the home,
//! the lists, the history, and the rail still read them (d-159a).
pub mod cross;
pub mod history;
pub mod labels;
pub mod list;
pub mod node;
pub mod now;
pub mod overview;
pub mod path;
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
    if let Some(answer) = static_answer(req, name) {
        return answer;
    }
    match req.path.as_str() {
        "/api/shell" => shell::shell(repo, req),
        "/api/now" => now::answer(repo, req),
        "/api/history" => history::answer(repo, req),
        "/api/labels" => labels::labels(repo, req),
        "/api/list" => list::rows(repo, req),
        "/api/overview" => overview::answer(repo),
        "/api/cross" => cross::answer(repo, req),
        "/api/search" => search::search(repo, req),
        "/api/ticks" => ticks::ticks(repo, req),
        _ => match req.path.strip_prefix("/api/node/") {
            Some(rest) => match rest.strip_suffix("/path") {
                Some(id) => path::answer(repo, &crate::http::decode(id, false), req),
                None => node::node(repo, &crate::http::decode(rest, false)),
            },
            None => Response::not_found(),
        },
    }
}

/// The pages and the assets, which need no ledger: the server answers them
/// before it opens one, so a browser's asset reads do not each pay for the
/// ledger (d-6d90). `None` is every other path.
pub fn static_answer(req: &Request, name: &str) -> Option<Response> {
    match req.path.as_str() {
        "/" | "/index.html" => Some(index(req, name)),
        path if path.starts_with("/api/") => None,
        _ => Some(static_file(req)),
    }
}

/// Whether a path is one of the reading questions the server answers once per
/// identical ask (d-6d90, ac-0dbb): the three screens' own, one node, and the
/// answers the older pages read.
pub fn is_reading(path: &str) -> bool {
    matches!(
        path,
        "/api/overview" | "/api/cross" | "/api/now" | "/api/list" | "/api/history" | "/api/labels"
    ) || path.starts_with("/api/node/")
}

/// The first HTML; the language comes from `Accept-Language` (ac-b9b1) and the
/// tab's title from the caller (n-07f0).
fn index(req: &Request, name: &str) -> Response {
    let ja = req
        .header("accept-language")
        .is_some_and(|value| value.trim_start().starts_with("ja"));
    let ledger = escape_html(name);
    let title = if name.is_empty() {
        "gy".to_string()
    } else {
        format!("gy - {}", ledger)
    };
    /* The fixed words first, the name last: a name cannot be caught by a later
    replacement (a directory named `%LANG%` stays itself). Both name slots
    are filled in one pass (n-f500). */
    let html = assets::index()
        .replace("%LANG%", if ja { "ja" } else { "en" })
        .replace("%REPO%", env!("CARGO_PKG_REPOSITORY"));
    Response::html(fill_names(&html, &ledger, &title))
}

/// The sidebar's name (`%LEDGER%`) and the tab's (`%NAME%`) in one pass over
/// the source: text just inserted is never read again, so a name may spell
/// either placeholder and still show itself (n-f500).
fn fill_names(html: &str, ledger: &str, title: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        if let Some(after) = tail.strip_prefix("%LEDGER%") {
            out.push_str(ledger);
            rest = after;
        } else if let Some(after) = tail.strip_prefix("%NAME%") {
            out.push_str(title);
            rest = after;
        } else {
            out.push('%');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
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
