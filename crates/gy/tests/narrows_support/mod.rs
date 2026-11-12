#![allow(dead_code)]
use crate::common::{Fixture, fixture, id_of, stderr, stdout};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Output, Stdio};
use std::time::Duration;

pub const MARKS: [&str; 3] = [
    "Context passage alpha.",
    "Decision passage beta.",
    "Scope passage gamma.",
];
pub const BODY: &str =
    "## Context\nContext passage alpha.\n\n## Decision\nDecision passage beta.\n";

pub fn ok(out: Output) -> Output {
    assert!(out.status.success(), "{} {}", stdout(&out), stderr(&out));
    out
}

pub fn write(fx: &Fixture, args: &[&str]) -> String {
    id_of(&ok(fx.run(args)))
}

pub fn setup(related: bool) -> (Fixture, String, String) {
    let fx = fixture();
    std::fs::write(fx.root.join("gy.toml"), "output = \"pub\"\n[scopes.a]\n").unwrap();
    let ac = write(&fx, &["criterion", "add", "measure"]);
    let need = write(&fx, &["need", "add", "need", "--targets", &ac]);
    let body = fx.root.join("body.md");
    std::fs::write(&body, BODY).unwrap();
    let y = write(
        &fx,
        &[
            "decide",
            "older",
            "--scope-note",
            MARKS[2],
            "--serves",
            &need,
            "--body-file",
            body.to_str().unwrap(),
        ],
    );
    let mut args = vec!["decide", "newer", "--scope-note", "here", "--serves", &need];
    if related {
        args.extend(["--relate", "narrows", &y, "--mark", MARKS[0]]);
    }
    let x = write(&fx, &args);
    (fx, x, y)
}

pub fn link(fx: &Fixture, x: &str, y: &str, mark: &str) {
    ok(fx.run(&["link", x, "narrows", y, "--mark", mark]));
}

pub fn three(fx: &Fixture, x: &str, y: &str, start: usize) {
    for mark in &MARKS[start..] {
        link(fx, x, y, mark);
    }
}

pub fn shown(fx: &Fixture, id: &str) -> Value {
    serde_json::from_slice::<Value>(&ok(fx.run(&["show", id, "--json"])).stdout).unwrap()[0].clone()
}

pub fn all_marks(node: &Value, x: &str) {
    assert_eq!(
        node["cancellation"]["narrowed"].as_array().unwrap().len(),
        3
    );
    for (i, mark) in MARKS.iter().enumerate() {
        let field = if i == 2 {
            "decision_scope_marked"
        } else {
            "body_marked"
        };
        assert!(
            node[field]
                .as_str()
                .unwrap()
                .contains(&format!("[[retracted by {x}: {mark}]]")),
            "{node}"
        );
        assert!(
            node["cancellation"]["narrowed"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["by"] == x && n["mark"] == *mark)
        );
    }
}

/// d-94bb: the frozen default may omit Context; every displayed passage must
/// be marked, while the summary must still name all three.
pub fn text_marks(text: &str, x: &str, legacy_default: bool) {
    for mark in MARKS {
        assert!(
            text.contains(&format!("narrowed-by {x} ({mark})")),
            "{text}"
        );
        if !legacy_default || mark != MARKS[0] || text.contains("## Context") {
            assert!(
                text.contains(&format!("[[retracted by {x}: {mark}]]")),
                "{text}"
            );
        }
    }
}

pub fn published(fx: &Fixture, x: &str, y: &str) {
    let page = std::fs::read_to_string(fx.root.join(format!("pub/a/{y}.md"))).unwrap();
    let source = std::fs::read_to_string(fx.root.join(format!("pub/a/{x}.md"))).unwrap();
    for mark in MARKS {
        assert!(
            page.contains(&format!("~~{mark}~~ *(retracted by {x})*")),
            "{page}"
        );
        assert!(source.contains(mark), "{source}");
    }
}

pub struct Server {
    child: Child,
    addr: String,
}
impl Server {
    pub fn start(fx: &Fixture, legacy: bool) -> Self {
        let mut command = if legacy {
            fx.compat_command(&["serve"])
        } else {
            fx.command(&["serve"])
        };
        let child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let mut server = Self {
            child,
            addr: String::new(),
        };
        let mut lines = BufReader::new(server.child.stdout.take().unwrap());
        let mut line = String::new();
        loop {
            assert!(lines.read_line(&mut line).unwrap() > 0, "serve exited");
            if let Some(at) = line.find("http://127.0.0.1:") {
                let addr = line[at..]
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .trim_end_matches('/')
                    .trim_start_matches("http://")
                    .to_string();
                server.addr = addr;
                return server;
            }
            line.clear();
        }
    }

    pub fn get(&self, path: &str) -> Value {
        let mut stream = TcpStream::connect(&self.addr).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            self.addr
        )
        .unwrap();
        let mut text = String::new();
        stream.read_to_string(&mut text).unwrap();
        let (header, body) = text.split_once("\r\n\r\n").unwrap();
        assert!(header.starts_with("HTTP/1.1 200"), "{text}");
        serde_json::from_str(body).unwrap()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
