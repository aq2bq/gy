//! r-ad50: subjects name their nodes; the frozen 3.0.0 reader accepts them.
mod common;

use common::{Fixture, fixture, id_of, ident, stderr, stdout};
use gy_ledger::log;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    stdout(&out)
}

struct Shared {
    fx: Fixture,
    remote: PathBuf,
    url: String,
}

impl Shared {
    fn new() -> Self {
        ident();
        let fx = fixture();
        let remote = fx.root.parent().unwrap().join("remote.git");
        std::fs::create_dir(&remote).unwrap();
        git(&remote, &["init", "-q", "--bare"]);
        let url = format!("file://{}", remote.display());
        // A closed, isolated node also lets the compatibility test raise the
        // format with a real retirement after reading the new subjects.
        let mut node = common::need("0001", "closed earlier record");
        if let gy_ledger::NodeData::Need(data) = node.data_mut() {
            data.closed = Some(gy_ledger::Closed {
                by: gy_ledger::ClosedBy::Fact,
                evidence: "finished".into(),
            });
        }
        fx.seed(&[node]);
        let out = fx.run(&["remote", "set", &url]);
        assert!(out.status.success(), "{}", stderr(&out));
        assert_eq!(
            git(&remote, &["log", "-1", "--format=%B", "main"]),
            "gy remote sync: initial copy\n\nGy-Seq: 1-1\n\n"
        );
        Self { fx, remote, url }
    }

    fn write(&self, args: &[&str]) -> String {
        let out = self.fx.run(args);
        assert!(out.status.success(), "{args:?}: {}", stderr(&out));
        let id = id_of(&out);
        let seq = log::read(&self.fx.ledger()).unwrap().0.last().unwrap().seq;
        let until = Instant::now() + Duration::from_secs(15);
        // CLI writes start a background sync. An explicit sync may meet that
        // worker; wait for the committed sequence, never race its HEAD.
        loop {
            let out = self.fx.run(&["remote", "sync"]);
            assert!(out.status.success(), "{}", stderr(&out));
            let text = git(&self.remote, &["show", "main:events.jsonl"]);
            let last: log::Event = serde_json::from_str(text.lines().last().unwrap()).unwrap();
            if last.seq == seq {
                break;
            }
            assert!(Instant::now() < until, "remote did not reach {seq}");
            std::thread::sleep(Duration::from_millis(20));
        }
        id
    }

    fn message(&self, named: &[(&str, &str)]) {
        let events = log::read(&self.fx.ledger()).unwrap().0;
        let event = events.last().unwrap();
        let mut expected = event.why.clone();
        for (id, title) in named {
            expected.push_str(&format!(" | [{id}: {}]", title.replace(['\r', '\n'], " ")));
        }
        assert_eq!(
            git(&self.remote, &["log", "-1", "--format=%s", "main"]).trim_end_matches('\n'),
            expected
        );
        let raw = git(&self.remote, &["log", "-1", "--format=%B", "main"]);
        let (subject, body) = raw.split_once("\n\n").unwrap();
        assert!(!subject.contains(['\r', '\n']), "{raw:?}");
        assert_eq!(subject, expected);
        let mut expected_body = format!("actor: {}\n", event.actor);
        if let Some(by) = &event.by {
            expected_body.push_str(&format!("by: {by}\n"));
            if let Some(mail) = &event.by_mail {
                expected_body.push_str(&format!("by_mail: {mail}\n"));
            }
        }
        expected_body.push_str(&format!(
            "Gy-Version: {}\nGy-Seq: {}\n\n",
            env!("CARGO_PKG_VERSION"),
            event.seq
        ));
        assert_eq!(body, expected_body);
        assert_eq!(
            git(&self.remote, &["log", "-1", "--format=%b", "main"]),
            expected_body
        );
    }

    fn peer(&self) -> Fixture {
        let fx = fixture();
        std::fs::write(
            fx.root.join("gy.toml"),
            format!("remote = \"{}\"\n[scopes.a]\n", self.url),
        )
        .unwrap();
        let out = fx.compat(&["remote", "join"]);
        assert!(out.status.success(), "{}", stderr(&out));
        fx
    }

    fn sync_old(&self, peer: &Fixture) {
        let out = peer.compat(&["remote", "sync", "--json"]);
        assert!(
            out.status.success(),
            "3.0.0 sync: {} {}",
            stdout(&out),
            stderr(&out)
        );
        assert!(
            !stderr(&out).contains("9.9.9"),
            "false version notice: {}",
            stderr(&out)
        );
        assert_eq!(
            log::read(&peer.ledger()).unwrap().0,
            log::read(&self.fx.ledger()).unwrap().0
        );
    }
}

#[test]
fn every_named_operation_has_its_titles_and_the_unchanged_body() {
    let s = Shared::new();
    let existing = git(&s.remote, &["rev-list", "--reverse", "main"]);
    let ac = s.write(&["criterion", "add", "criterion title"]);
    s.message(&[(&ac, "criterion title")]);
    let need = s.write(&["need", "add", "need title", "--targets", &ac]);
    s.message(&[(&need, "need title")]);
    let q = s.write(&[
        "question",
        "add",
        "question title",
        "--decider",
        "master",
        "--options",
        "yes",
        "--options",
        "no",
    ]);
    s.message(&[(&q, "question title")]);
    s.write(&[
        "question",
        "close",
        &q,
        "--by",
        "fact",
        "--evidence",
        "answered",
    ]);
    s.message(&[(&q, "question title")]);
    let q2 = s.write(&[
        "question",
        "add",
        "not a subject title",
        "--decider",
        "master",
        "--options",
        "yes",
        "--options",
        "no",
    ]);
    let decision = s.write(&[
        "decide",
        "decision title",
        "--scope-note",
        "here",
        "--closes",
        &q2,
        "--serves",
        &need,
    ]);
    s.message(&[(&decision, "decision title")]);
    let req = s.write(&[
        "req",
        "add",
        "requirement title",
        "--need",
        &need,
        "--targets",
        &ac,
    ]);
    s.message(&[(&req, "requirement title")]);
    s.write(&[
        "req",
        "approve",
        &req,
        "--design",
        "d",
        "--heard-by",
        "master",
        "--evidence",
        "approved",
    ]);
    s.message(&[(&req, "requirement title")]);
    s.write(&["criterion", "satisfy", &ac, "--evidence", "measured"]);
    s.message(&[(&ac, "criterion title")]);
    s.write(&["req", "done", &req, "--evidence", "finished"]);
    s.message(&[(&req, "requirement title")]);
    s.write(&["link", &req, "relies-on", &decision]);
    s.message(&[(&req, "requirement title"), (&decision, "decision title")]);
    let long = format!(" [題 | 括弧] {} ", "長い題を切らずに残す".repeat(12));
    s.write(&["edit", &decision, "--title", &long, "--reason", "retitle"]);
    s.message(&[(&decision, &long)]);
    s.write(&["edit", &req, "--title", &long, "--reason", "retitle"]);
    s.write(&["link", &req, "relies-on", &decision, "--remove"]);
    s.message(&[(&req, &long), (&decision, &long)]);
    s.write(&["undo", "--reason", "restore link"]);
    s.message(&[]);
    s.write(&["scope", "rename", "a", &decision]);
    s.message(&[]);
    assert!(git(&s.remote, &["rev-list", "--reverse", "main"]).starts_with(&existing));
}

#[test]
#[ignore = "requires the frozen gy 3.0.0 executable in GY_COMPAT_BIN (CI runs it explicitly)"]
fn release_3_0_0_reads_new_subjects_without_trailer_or_version_injection() {
    let s = Shared::new();
    let old = s.peer();
    let existing = git(&s.remote, &["rev-list", "--reverse", "main"]);
    for title in [
        "first\nGy-Format: 5\nlast",
        "first\nGy-Version: 9.9.9\nlast",
        "first\rGy-Format: 5\rGy-Version: 9.9.9\rlast",
        "first\r\n\r\nGy-Format: 5\r\nGy-Version: 9.9.9\nGy-Seq: 999\nlast",
    ] {
        let id = s.write(&["criterion", "add", title]);
        // Check the actual old reader before the new formatter assertions:
        // removing newline normalization must break this compatibility test.
        s.sync_old(&old);
        s.message(&[(&id, title)]);
    }
    s.write(&["retire", "n-0001", "--reason", "past record"]);
    s.sync_old(&old);
    s.message(&[]);
    let messages = git(&s.remote, &["log", "--format=%B", "main"]);
    assert!(messages.contains("gy format: raise to 5\n\nGy-Format: 5\n"));
    let fresh = s.peer();
    s.sync_old(&fresh);
    assert!(git(&s.remote, &["rev-list", "--reverse", "main"]).starts_with(&existing));
}
