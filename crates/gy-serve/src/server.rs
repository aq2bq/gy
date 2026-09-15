//! The one file where tiny_http appears (ac-a49a): bind, accept, conversions.
use crate::api;
use crate::http;
use gy_ledger::{Error, FileStore, Repository, Result};
use std::sync::Arc;
use tiny_http::{Header, Request, Response, Server};

/// The ports gy serve may take: the first that binds, 7331 first (ac-f5c4).
const FIRST_PORT: u16 = 7331;
const LAST_PORT: u16 = 7400;

/// How a request opens the ledger: afresh each time, so no cache hides a
/// write (n-a493). The CLI supplies it.
pub type Opener = Box<dyn Fn() -> Result<Repository<FileStore>> + Send + Sync>;

/// Serve until stopped, printing the one line the master needs to open.
pub fn serve(open: Opener) -> Result<()> {
    let (server, port) = bind()?;
    println!("gy serve  http://127.0.0.1:{port}/");
    let open = Arc::new(open);
    for request in server.incoming_requests() {
        let open = open.clone();
        std::thread::spawn(move || handle(&open, request));
    }
    Ok(())
}

/// The first port of the range that binds. tiny_http refuses a taken port, so
/// this needs no probe (d-8a24).
pub fn bind() -> Result<(Server, u16)> {
    let mut refused = String::new();
    for port in FIRST_PORT..=LAST_PORT {
        match Server::http(("127.0.0.1", port)) {
            Ok(server) => return Ok((server, port)),
            Err(error) => refused = error.to_string(),
        }
    }
    Err(Error::invalid(format!(
        "no free port in {FIRST_PORT}..={LAST_PORT}: {refused}"
    )))
}

/// tiny_http's request in, tiny_http's response out; the route sees neither.
fn handle(open: &Opener, request: Request) {
    let session = convert(&request);
    let answer = match open() {
        Ok(repo) => api::route(&repo, &session),
        Err(error) => http::Response::text(500, &error.to_string()),
    };
    let header = Header::from_bytes("Content-Type", answer.content_type).expect("static header");
    let _ = request.respond(
        Response::from_data(answer.body)
            .with_status_code(answer.status)
            .with_header(header),
    );
}

/// The request shape `crate::http` defines, from tiny_http's request.
fn convert(request: &Request) -> http::Request {
    let url = request.url();
    let (path, query) = match url.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (url, None),
    };
    let mut headers = Vec::new();
    for header in request.headers() {
        let field = header.field.as_str().as_str().to_string();
        let value = header.value.as_str().to_string();
        headers.push((field, value));
    }
    http::Request::new(request.method().as_str(), path, query, &headers)
}
