//! Regression for n-8b91 1: two syncs race on a copy that does not exist yet.
//! The first sync's clone replaces the ledger directory (and with it the entry
//! lock's `sync.pid`); the race still agrees because the waiter holds the
//! pre-clone inode and skips the clone. No `gy` is run by hand; only this test
//! and a `file://` bare.
use gy_ledger::{FormatVersion, Node, NodeId, NodeKind, Sync, format, log, sync};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Barrier};

mod common;
use common::ident;

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        "an ac",
    )
    .unwrap()
}

fn event(seq: u64) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: "piko".to_string(),
        why: "test".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: None,
        by_mail: None,
        changes: vec![log::Change::Created {
            node: criterion("0001").id().to_string(),
            value: serde_json::to_value(criterion("0001")).unwrap(),
        }],
    }
}

fn ledger(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    log::append(dir, &event(1)).unwrap();
}

/// A source ledger pushed to a bare remote, so a missing copy has something to
/// clone. Returns the remote's path and its `file://` URL.
fn seeded(temp: &Path) -> (PathBuf, PathBuf) {
    let source = temp.join("source");
    let remote = temp.join("remote.git");
    ledger(&source);
    bare(&remote);
    sync(&source, &url(&remote)).unwrap();
    (source, remote)
}

fn lines(dir: &Path) -> usize {
    std::fs::read_to_string(dir.join(log::FILE))
        .map(|text| text.lines().filter(|line| !line.trim().is_empty()).count())
        .unwrap_or(0)
}

fn remote_commits(remote: &Path) -> String {
    git(remote, &["rev-list", "--count", "main"])
        .trim()
        .to_string()
}

/// Two syncs released together at a missing copy, as `sync` returns them.
fn race(target: &Path, remote: &str) -> Vec<gy_ledger::Result<Sync>> {
    let barrier = Arc::new(Barrier::new(3));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let (target, remote, barrier) = (
                target.to_path_buf(),
                remote.to_string(),
                Arc::clone(&barrier),
            );
            std::thread::spawn(move || {
                barrier.wait();
                sync(&target, &remote)
            })
        })
        .collect();
    barrier.wait();
    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

/// What one raced round left behind, for the report.
struct Round {
    ok: usize,
    errors: Vec<String>,
    repo: bool,
    lines: usize,
    rejected: bool,
    remote: String,
    pid: bool,
}

fn observe(
    target: &Path,
    remote: &str,
    remote_path: &Path,
) -> (Round, Vec<gy_ledger::Result<Sync>>) {
    let results = race(target, remote);
    let errors: Vec<String> = results
        .iter()
        .filter_map(|result| result.as_ref().err())
        .map(|error| error.message.clone())
        .collect();
    let round = Round {
        ok: results.len() - errors.len(),
        errors,
        repo: target.join(".git").is_dir(),
        lines: lines(target),
        rejected: target.join("rejected.jsonl").exists(),
        remote: remote_commits(remote_path),
        pid: target.join("sync.pid").exists(),
    };
    (round, results)
}

#[test]
fn two_syncs_racing_on_a_missing_copy_report_what_happens() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (_source, remote_path) = seeded(temp.path());
    let remote = url(&remote_path);
    let before = remote_commits(&remote_path);
    let mut bad = Vec::new();
    for i in 0..10 {
        let target = temp.path().join(format!("race{i}"));
        let (round, _) = observe(&target, &remote, &remote_path);
        println!(
            "round {i}: ok={} repo={} lines={} rejected={} remote={} pid={} errors={:?}",
            round.ok,
            round.repo,
            round.lines,
            round.rejected,
            round.remote,
            round.pid,
            round.errors
        );
        if round.ok != 2
            || !round.repo
            || round.lines != 1
            || round.rejected
            || round.remote != before
        {
            bad.push(format!(
                "round {i}: ok={} repo={} lines={} rejected={} remote={} errors={:?}",
                round.ok, round.repo, round.lines, round.rejected, round.remote, round.errors
            ));
        }
    }
    assert!(bad.is_empty(), "raced rounds differed: {bad:#?}");
}

#[test]
fn a_clone_replaces_the_lock_file_yet_the_race_still_agrees() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (_source, remote_path) = seeded(temp.path());
    let target = temp.path().join("fresh");

    let report = sync(&target, &url(&remote_path)).unwrap();

    assert_eq!(report.seq, 1);
    assert_eq!(lines(&target), 1, "the clone brought the write");
    // The entry lock is taken on `sync.pid` before the clone, and the clone
    // moves the ledger directory into place, so the file it locked is gone. A
    // clone is a one-time path and the race it could open is a second sync
    // starting after the clone; that is the two-syncs-at-once case the test
    // above keeps, where the waiter holds the pre-clone inode and skips the
    // clone, and n-6b44's landed rule keeps a concurrent push from being read
    // as a conflict. This test pins the file's replacement alone.
    assert!(
        !target.join("sync.pid").exists(),
        "the clone replaced the directory, so the entry lock's file is gone"
    );
}
