//! The HTTP layer, on the standard library alone (d-e6c4): a bound listener,
//! one thread per connection, and the two conversions. GET only; the request
//! line and the headers are read, the body never is.
use crate::api;
use crate::cache::{Flights, LIMIT};
use crate::http;
use crate::watch::Watch;
use gy_ledger::{Error, FileStore, MemoryStore, Repository, Result};
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

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

/// Serve until stopped, printing the one line a person needs to open. The
/// watch follows `ledger`'s log so a request can wait for the next write. The
/// name is the ledger directory's, for the tab's title (n-07f0).
pub fn serve(open: Opener, ledger: PathBuf, name: String, remote: Option<String>) -> Result<()> {
    let (listener, port) = bind()?;
    let url = format!("http://127.0.0.1:{port}/");
    // One line for the terminal: where it listens, what it reads, and whether
    // it syncs (n-94bb).
    eprintln!(
        "listening on {url}  ledger {}  remote {}",
        ledger.display(),
        remote.as_deref().unwrap_or("none")
    );
    println!("gy serve  {url}");
    if std::io::stdout().is_terminal() {
        open_browser(&url);
    }
    run(listener, open, ledger, name)
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
pub fn run(listener: TcpListener, open: Opener, ledger: PathBuf, name: String) -> Result<()> {
    let open = Arc::new(open);
    let name = Arc::new(name);
    let watch = Watch::start(ledger.clone());
    let kept = Arc::new(Ledger::new(&ledger));
    let flights = Flights::default();
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let open = open.clone();
        let name = name.clone();
        let watch = watch.clone();
        let kept = kept.clone();
        let flights = flights.clone();
        std::thread::spawn(move || {
            let _ = connection(
                open.clone(),
                &watch,
                kept.clone(),
                &flights,
                name.clone(),
                stream,
            );
        });
    }
    Ok(())
}

/// The log's length and time: the two things every write changes, and the check
/// the watcher already makes (d-8a24).
type Stamp = (u64, Option<SystemTime>);

/// The opened ledger, kept from one request to the next while the log has not
/// changed (d-6d90). Opening the log costs most of a small answer on a large
/// record — more than any of the reading questions — and a write changes the
/// log, so the stamp is what says a kept ledger is still the head. A request
/// for a point in the log (`at`) is opened fresh, since the scrubber may ask
/// for any point in any order.
pub struct Ledger {
    /// The log to stamp; `None` keeps nothing (the tests that answer directly).
    file: Option<PathBuf>,
    head: Mutex<Option<(Stamp, Arc<Opened>)>>,
}

impl Ledger {
    pub fn new(dir: &Path) -> Self {
        Self {
            file: Some(dir.join(gy_ledger::log::FILE)),
            head: Mutex::new(None),
        }
    }

    /// A ledger that keeps nothing: every open is the caller's own.
    fn uncached() -> Self {
        Self {
            file: None,
            head: Mutex::new(None),
        }
    }

    /// The ledger for a point: the kept head while the log stands still, and a
    /// fresh open otherwise.
    fn read(&self, open: &Opener, at: Option<u64>) -> Result<Arc<Opened>> {
        let Some(file) = &self.file else {
            return open(at).map(Arc::new);
        };
        if at.is_some() {
            return open(at).map(Arc::new);
        }
        let stamp = stamp(file);
        if let Some((held, opened)) = self.head.lock().unwrap().as_ref() {
            if *held == stamp {
                return Ok(Arc::clone(opened));
            }
        }
        let opened = Arc::new(open(None)?);
        *self.head.lock().unwrap() = Some((stamp, Arc::clone(&opened)));
        Ok(opened)
    }
}

/// The log's length and modified time, or zeros when it is not there yet.
fn stamp(file: &Path) -> Stamp {
    std::fs::metadata(file).map_or((0, None), |meta| (meta.len(), meta.modified().ok()))
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
    open: Arc<Opener>,
    watch: &Watch,
    kept: Arc<Ledger>,
    flights: &Flights,
    name: Arc<String>,
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
    let answer = if let Some(answer) = api::static_answer(&session, &name) {
        answer
    } else if session.path == "/api/wait" {
        api::wait::wait(watch, &session)
    } else if api::is_reading(&session.path) {
        /* A reading question is computed once, and a second ask for it waits
        (d-6d90, ac-0dbb): the worker owns the ledger and the answer. */
        let key = flight_key(&session, watch.current());
        let request = session.clone();
        let kept = Arc::clone(&kept);
        flights.answer(&key, LIMIT, move || {
            answer_kept(&kept, &open, &request, &name)
        })
    } else {
        answer_kept(&kept, &open, &session, &name)
    };
    respond(stream, answer)
}

/// The one name of a reading question: its path, its query in a fixed order,
/// and the last write the watcher has seen — so a request that follows a write
/// never rides the computation of the ledger before it.
fn flight_key(session: &http::Request, seen: u64) -> String {
    let mut query: Vec<&(String, String)> = session.query.iter().collect();
    query.sort();
    let query: Vec<String> = query
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    format!("{}?{}@{seen}", session.path, query.join("&"))
}

/// The answer to one framework-free request: `at` picks the point in the log,
/// and a broken one is 400. The scrubber always sees the whole log, so
/// `/api/ticks` ignores `at` (n-10e1).
pub fn answer(open: &Opener, session: &http::Request, name: &str) -> http::Response {
    answer_kept(&Ledger::uncached(), open, session, name)
}

/// The same, from the kept head when it is still the head.
fn answer_kept(
    kept: &Ledger,
    open: &Opener,
    session: &http::Request,
    name: &str,
) -> http::Response {
    let at = match requested_at(session) {
        Ok(at) => at,
        Err(response) => return response,
    };
    match kept.read(open, at) {
        Ok(opened) => match &*opened {
            Opened::Now(repo) => api::route(repo, session, name),
            Opened::At(repo) => api::route(repo, session, name),
        },
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
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}
