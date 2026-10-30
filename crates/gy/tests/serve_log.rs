//! ac-0dbb (3): a reading answer that fails writes one line to gy serve's log.
//! The log is stderr, so the test starts the real server, asks a question the
//! broken ledger cannot answer, and reads what the server printed (r-44c7).
mod common;

use common::fixture;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

/// A ledger with one write, then a log line no event can be read from: the
/// server still starts (it only needs the log to exist), but a request that
/// opens the ledger fails.
fn broken_ledger() -> common::Fixture {
    let fx = fixture();
    let written = fx.run(&["criterion", "add", "a check"]);
    assert!(written.status.success(), "{}", common::stderr(&written));
    let data = fx.data.join("gy");
    let ledger = std::fs::read_dir(&data)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
        .expect("the ledger directory");
    let mut log = std::fs::OpenOptions::new()
        .append(true)
        .open(ledger.join("events.jsonl"))
        .unwrap();
    writeln!(log, "not an event").unwrap();
    fx
}

/// The server, its URL, and the two pipes.
fn serve(fx: &common::Fixture) -> (Child, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env_remove("GY_ACTOR")
        .arg("serve")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut said = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut said)
        .unwrap();
    let url = said
        .split_whitespace()
        .find(|word| word.starts_with("http://"))
        .unwrap_or_else(|| panic!("the server printed no url: {said}"))
        .to_string();
    (child, url)
}

/// One GET's status line, and the server closes the connection after it.
fn ask(url: &str, path: &str) -> String {
    let host = url
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string();
    let mut stream = TcpStream::connect(&host).unwrap();
    write!(stream, "GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n").unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    answer.lines().next().unwrap_or_default().to_string()
}

/// The one line the server wrote about `path`.
fn lines_about(log: &str, path: &str) -> Vec<String> {
    log.lines()
        .filter(|line| line.contains(path))
        .map(str::to_string)
        .collect()
}

#[test]
fn a_failed_answer_writes_one_line_to_the_log() {
    let fx = broken_ledger();
    let (mut child, url) = serve(&fx);
    let status = ask(&url, "/api/overview");
    let _ = child.kill();
    let output = child.wait_with_output().unwrap();
    let log = String::from_utf8_lossy(&output.stderr).into_owned();

    assert!(status.contains(" 500 "), "the answer was {status:?}");
    let lines = lines_about(&log, "/api/overview");
    assert_eq!(lines.len(), 1, "the log said {log:?}");
    assert!(lines[0].contains("500"), "the line was {:?}", lines[0]);
}

/// A question that answers writes nothing but its arrival line: the log stays
/// quiet, so a reader is not sent looking for a failure that did not happen.
#[test]
fn an_answered_question_writes_no_failure_line() {
    let fx = fixture();
    let written = fx.run(&["criterion", "add", "a check"]);
    assert!(written.status.success(), "{}", common::stderr(&written));
    let (mut child, url) = serve(&fx);
    let status = ask(&url, "/api/overview");
    let _ = child.kill();
    let output = child.wait_with_output().unwrap();
    let log = String::from_utf8_lossy(&output.stderr).into_owned();

    assert!(status.contains(" 200 "), "the answer was {status:?}");
    assert!(lines_about(&log, "no answer").is_empty(), "{log:?}");
    assert!(lines_about(&log, "/api/overview:").is_empty(), "{log:?}");
}
