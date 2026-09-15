//! The HTTP layer, on the standard library alone (d-e6c4): a bound listener,
//! one thread per connection, and the two conversions. GET only; the request
//! line and the headers are read, the body never is.
use crate::api;
use crate::graph_cache::GraphCache;
use crate::http;
use crate::watch::Watch;
use gy_ledger::{Error, FileStore, MemoryStore, Repository, Result};
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

/// The ports gy serve may take: the first that binds, 7331 first (ac-f5c4).
const FIRST_PORT: u16 = 7331;
const LAST_PORT: u16 = 7400;
/// A connection may take this long to send its request line and headers.
const READ_TIMEOUT: Duration = Duration::from_secs(5);
/// The most a request's headers may hold before it is refused (431).
const HEAD_LIMIT: usize = 64 * 1024;

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
    let (listener, port) = bind()?;
    let url = format!("http://127.0.0.1:{port}/");
    println!("gy serve  {url}");
    if std::io::stdout().is_terminal() {
        open_browser(&url);
    }
    run(listener, open, ledger)
}

/// Open the URL once, without waiting; a failure only prints one line and the
/// server keeps running (d-5b3d). Only called when stdout is a terminal.
fn open_browser(url: &str) {
    let opened = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(url)
            .spawn()
    } else {
        Command::new("xdg-open").arg(url).spawn()
    };
    if let Err(error) = opened {
        eprintln!("could not open a browser: {error}");
    }
}

/// The same, on a listener the caller already holds (a test wants the port).
pub fn run(listener: TcpListener, open: Opener, ledger: PathBuf) -> Result<()> {
    let open = Arc::new(open);
    let watch = Watch::start(ledger);
    let cache = GraphCache::new();
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let open = open.clone();
        let watch = watch.clone();
        let cache = cache.clone();
        std::thread::spawn(move || {
            let _ = connection(&open, &watch, &cache, stream);
        });
    }
    Ok(())
}

/// The first port of the range that binds. A taken port is refused, so this
/// needs no probe (d-8a24).
pub fn bind() -> Result<(TcpListener, u16)> {
    let mut refused = String::new();
    for port in FIRST_PORT..=LAST_PORT {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => return Ok((listener, port)),
            Err(error) => refused = error.to_string(),
        }
    }
    Err(Error::invalid(format!(
        "no free port in {FIRST_PORT}..={LAST_PORT}: {refused}"
    )))
}

/// One connection: read the request, answer, close. A failure here ends this
/// thread only.
fn connection(
    open: &Opener,
    watch: &Watch,
    cache: &GraphCache,
    mut stream: TcpStream,
) -> std::io::Result<()> {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let session = match read_request(&mut stream) {
        Read::Request(request) => request,
        Read::Refuse(status) => {
            return respond(stream, http::Response::text(status, reason(status)));
        }
        Read::Silent => return Ok(()),
    };
    let answer = if session.path == "/api/wait" {
        api::wait::wait(watch, &session)
    } else if session.path == "/api/graph" {
        match requested_at(&session) {
            Ok(at) => api::graph::answer(open, cache, at, &session),
            Err(response) => response,
        }
    } else {
        answer(open, &session)
    };
    respond(stream, answer)
}

/// The answer to one framework-free request: `at` picks the point in the log,
/// and a broken one is 400. The scrubber always sees the whole log, so
/// `/api/ticks` ignores `at` (n-10e1).
pub fn answer(open: &Opener, session: &http::Request) -> http::Response {
    let at = match requested_at(session) {
        Ok(at) => at,
        Err(response) => return response,
    };
    match open(at) {
        Ok(Opened::Now(repo)) => api::route(&repo, session),
        Ok(Opened::At(repo)) => api::route(&repo, session),
        Err(error) => http::Response::text(500, &error.to_string()),
    }
}

/// The point in the log a request asks for: `at` when it is there and sound.
/// The scrubber always sees the whole log, so `/api/ticks` ignores it.
pub fn requested_at(session: &http::Request) -> std::result::Result<Option<u64>, http::Response> {
    if session.path == "/api/ticks" {
        return Ok(None);
    }
    match session.param("at") {
        Some(text) => match text.parse::<u64>() {
            Ok(seq) => Ok(Some(seq)),
            Err(_) => Err(http::Response::text(400, "at is not a sequence")),
        },
        None => Ok(None),
    }
}

/// What the reader came back with.
enum Read {
    Request(http::Request),
    Refuse(u16),
    /// The peer left, or said nothing this server can answer: close quietly.
    Silent,
}

/// Read the request line and the headers of one GET. The body is left in the
/// socket, since a GET has none and this server closes the connection.
fn read_request(stream: &mut TcpStream) -> Read {
    let mut reader = BufReader::new(&mut *stream);
    let Some(line) = read_line(&mut reader) else {
        return Read::Silent;
    };
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 || !matches!(parts[2], "HTTP/1.0" | "HTTP/1.1") {
        return Read::Refuse(400);
    }
    let headers = match read_headers(&mut reader, line.len()) {
        Headers::Given(headers) => headers,
        Headers::TooLarge => return Read::Refuse(431),
        Headers::Gone => return Read::Silent,
    };
    if parts[0] != "GET" {
        return Read::Refuse(405);
    }
    let (path, query) = match parts[1].split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (parts[1], None),
    };
    Read::Request(http::Request::new(parts[0], path, query, &headers))
}

/// One line of text, without its newline.
fn read_line(reader: &mut impl BufRead) -> Option<String> {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim_end().to_string()),
    }
}

/// What reading the headers came back with.
enum Headers {
    Given(Vec<(String, String)>),
    /// The headers grew past the limit: 431.
    TooLarge,
    /// The peer left, or stopped sending: say nothing.
    Gone,
}

/// The headers up to the blank line, with the names lowered and the values
/// trimmed, as `http::Request` takes them.
fn read_headers(reader: &mut impl BufRead, mut held: usize) -> Headers {
    let mut headers = Vec::new();
    loop {
        let Some(line) = read_line(reader) else {
            return Headers::Gone;
        };
        held += line.len() + 2;
        if held > HEAD_LIMIT {
            return Headers::TooLarge;
        }
        if line.is_empty() {
            return Headers::Given(headers);
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
        }
    }
}

/// Write the answer and close: the length is known, so no chunking is needed.
fn respond(mut stream: TcpStream, answer: http::Response) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        answer.status,
        reason(answer.status),
        answer.content_type,
        answer.body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&answer.body)?;
    stream.flush()
}

/// The reason phrase of every status this server answers with.
fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}
