use gy_ledger::{Actor, FormatVersion, MemoryStore, Repository};
use gy_serve::server::{Opened, Opener, bind, run};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

/// Hold the first free port of the range, then bind again: the server must take
/// a different port (7331 is not assumed free).
#[test]
fn bind_skips_a_bound_port() {
    let held = (7331..=7400u16).find_map(|port| TcpListener::bind(("127.0.0.1", port)).ok());
    let held = held.expect("a free port in the range");
    let (server, port) = bind().unwrap();
    assert_ne!(port, held.local_addr().unwrap().port());
    drop(server);
}

/// A server on its own port with an empty in-memory ledger, so the pages
/// answer without a disk.
fn start() -> u16 {
    let open: Opener = Box::new(|_| {
        let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("test").unwrap());
        Ok(Opened::At(Repository::new(store)))
    });
    let dir = std::env::temp_dir().join(format!("gy-serve-http-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let (listener, port) = bind().unwrap();
    thread::spawn(move || {
        let _ = run(listener, open, dir, "gy".to_string());
    });
    port
}

/// One exchange: send the bytes, then read the answer to EOF (the server
/// closes every connection).
fn exchange(port: u16, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(8)))
        .unwrap();
    let _ = stream.write_all(request);
    let mut answer = Vec::new();
    let _ = stream.read_to_end(&mut answer);
    String::from_utf8_lossy(&answer).into_owned()
}

fn get(port: u16, path: &str) -> String {
    exchange(
        port,
        format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes(),
    )
}

/// The head of an answer, up to the body.
fn head(answer: &str) -> &str {
    answer.split("\r\n\r\n").next().unwrap_or(answer)
}

fn body(answer: &str) -> &str {
    answer.split_once("\r\n\r\n").map_or("", |(_, body)| body)
}

#[test]
fn a_get_answers_with_a_length_and_closes() {
    let port = start();
    let answer = get(port, "/");
    assert!(answer.starts_with("HTTP/1.1 200 OK\r\n"), "{answer}");
    assert!(head(&answer).contains("Connection: close"), "{answer}");
    assert!(
        head(&answer).contains("Cache-Control: no-store"),
        "{answer}"
    );

    let declared: usize = head(&answer)
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .expect("a content length")
        .parse()
        .unwrap();
    assert_eq!(declared, body(&answer).len());
    assert!(body(&answer).contains("<html"), "the index did not come");
}

#[test]
fn a_post_is_not_allowed_and_the_next_get_is_fine() {
    let port = start();
    let answer = exchange(port, b"POST /api/shell HTTP/1.1\r\nHost: x\r\n\r\n");
    assert!(answer.starts_with("HTTP/1.1 405 "), "{answer}");
    assert!(get(port, "/").starts_with("HTTP/1.1 200 "));
}

#[test]
fn a_broken_request_line_is_refused() {
    let port = start();
    for request in [
        b"\r\n\r\n".as_slice(),
        b"GARBAGE\r\n\r\n",
        b"PRI * HTTP/2.0\r\n\r\n",
    ] {
        let answer = exchange(port, request);
        assert!(answer.starts_with("HTTP/1.1 400 "), "{answer}");
        assert!(get(port, "/").starts_with("HTTP/1.1 200 "));
    }
}

#[test]
fn a_header_flood_is_refused() {
    let port = start();
    let flood = format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(70 * 1024));
    let answer = exchange(port, flood.as_bytes());
    assert!(answer.starts_with("HTTP/1.1 431 "), "{}", head(&answer));
    assert!(get(port, "/").starts_with("HTTP/1.1 200 "));
}

#[test]
fn a_silent_connection_is_dropped_at_the_timeout() {
    let port = start();
    let started = Instant::now();
    let answer = exchange(port, b"");
    assert!(answer.is_empty(), "{answer}");
    assert!(started.elapsed() >= Duration::from_secs(4));
    assert!(started.elapsed() < Duration::from_secs(8));
    assert!(get(port, "/").starts_with("HTTP/1.1 200 "));
}

#[test]
fn a_tls_hello_is_refused_or_closed() {
    let port = start();
    let answer = exchange(port, &[0x16, 0x03, 0x01, 0x00, 0x05, 0x01, 0x02, 0x03]);
    assert!(
        answer.is_empty() || answer.starts_with("HTTP/1.1 400 "),
        "{answer}"
    );
    assert!(get(port, "/").starts_with("HTTP/1.1 200 "));
}

#[test]
fn fifteen_connections_at_once_all_answer() {
    let port = start();
    let handles: Vec<_> = (0..15)
        .map(|_| thread::spawn(move || get(port, "/")))
        .collect();
    for handle in handles {
        let answer = handle.join().unwrap();
        assert!(answer.starts_with("HTTP/1.1 200 "), "{answer}");
    }
}
