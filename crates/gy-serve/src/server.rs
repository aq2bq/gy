//! The one file where tiny_http appears (ac-a49a): bind, accept, conversions.
use crate::api;
use crate::http;
use crate::watch::Watch;
use gy_ledger::{Error, FileStore, MemoryStore, Repository, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tiny_http::{Header, Request, Response, Server};

/// The ports gy serve may take: the first that binds, 7331 first (ac-f5c4).
const FIRST_PORT: u16 = 7331;
const LAST_PORT: u16 = 7400;

/// The ledger a request reads: the head, or a point in the log (n-10e1).
pub enum Opened {
    Now(Repository<FileStore>),
    At(Repository<MemoryStore>),
}

/// How a request opens the ledger: afresh each time, so no cache hides a
/// write (n-a493), at the asked sequence when one is given (n-10e1).
pub type Opener = Box<dyn Fn(Option<u64>) -> Result<Opened> + Send + Sync>;

/// Serve until stopped, printing the one line the master needs to open. The
/// watch follows `ledger`'s log so a request can wait for the next write.
pub fn serve(open: Opener, ledger: PathBuf) -> Result<()> {
    let (server, port) = bind()?;
    println!("gy serve  http://127.0.0.1:{port}/");
    let open = Arc::new(open);
    let watch = Watch::start(ledger);
    for request in server.incoming_requests() {
        let open = open.clone();
        let watch = watch.clone();
        std::thread::spawn(move || handle(&open, &watch, request));
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

/// The answer to one framework-free request: `at` picks the point in the log,
/// and a broken one is 400. The scrubber always sees the whole log, so
/// `/api/ticks` ignores `at` (n-10e1).
pub fn answer(open: &Opener, session: &http::Request) -> http::Response {
    let at = if session.path == "/api/ticks" {
        None
    } else {
        match session.param("at") {
            Some(text) => match text.parse::<u64>() {
                Ok(seq) => Some(seq),
                Err(_) => return http::Response::text(400, "at is not a sequence"),
            },
            None => None,
        }
    };
    match open(at) {
        Ok(Opened::Now(repo)) => api::route(&repo, session),
        Ok(Opened::At(repo)) => api::route(&repo, session),
        Err(error) => http::Response::text(500, &error.to_string()),
    }
}

/// tiny_http's request in, tiny_http's response out; the route sees neither.
/// Waiting for the next write is the one route that does not touch the ledger.
fn handle(open: &Opener, watch: &Watch, request: Request) {
    let session = convert(&request);
    let answer = if session.path == "/api/wait" {
        api::wait::wait(watch, &session)
    } else {
        answer(open, &session)
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
