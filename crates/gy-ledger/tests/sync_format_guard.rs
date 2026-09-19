//! The guard admits a format-only commit and refuses the rest (n-96f8,
//! ac-682e): a lower version, another file beside `format`, or both the
//! `Gy-Format` and the `Gy-Seq` trailer. Local bare remotes only.
use gy_ledger::{FormatVersion, Node, NodeId, NodeKind, format, log, sync};
use std::path::Path;
use std::process::Command;

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

fn event(seq: u64, changes: Vec<log::Change>) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: "piko".to_string(),
        why: "a write".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: None,
        by_mail: None,
        changes,
    }
}

fn created(node: &Node) -> log::Change {
    log::Change::Created {
        node: node.id().to_string(),
        value: serde_json::to_value(node).unwrap(),
    }
}

/// A ledger at `version` with one write, pushed later by `sync`. Writing the
/// file directly keeps the copy at `version`, because sync does not open it.
fn seed(dir: &Path, version: FormatVersion) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, version).unwrap();
    log::append(dir, &event(1, vec![created(&criterion("0001"))])).unwrap();
}

/// The first copy and its remote, both at `version`; returns a second copy
/// cloned from the same remote, ready to make a hand commit.
fn pair(temp: &Path, version: FormatVersion) -> (std::path::PathBuf, std::path::PathBuf) {
    let remote = temp.join("remote.git");
    let first = temp.join("first");
    bare(&remote);
    seed(&first, version);
    sync(&first, &url(&remote)).unwrap();
    let second = temp.join("second");
    sync(&second, &url(&remote)).unwrap();
    (first, second)
}

/// Make one hand commit on `copy` and push it to the remote's `main`.
fn hand_commit(copy: &Path, message: &str) {
    git(copy, &["add", "-A"]);
    git(copy, &["commit", "-q", "-m", message]);
    git(copy, &["push", "-q", "origin", "HEAD:main"]);
}

#[test]
fn refuses_a_format_commit_that_lowers_the_version() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second) = pair(temp.path(), FormatVersion::CURRENT);

    std::fs::write(second.join(format::FILE), "3\n").unwrap();
    hand_commit(&second, "lower\n\nGy-Format: 3");

    let error = sync(&first, &url(&temp.path().join("remote.git"))).unwrap_err();
    assert!(
        error.message.contains("lowers the format"),
        "{}",
        error.message
    );
    // The refused take-in leaves this copy's own format alone.
    assert_eq!(
        std::fs::read_to_string(first.join(format::FILE)).unwrap(),
        "4\n"
    );
}

#[test]
fn refuses_a_format_commit_that_touches_another_file() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second) = pair(temp.path(), FormatVersion(3));

    std::fs::write(second.join(format::FILE), "4\n").unwrap();
    std::fs::write(second.join("notes.txt"), "hello").unwrap();
    hand_commit(&second, "raise\n\nGy-Format: 4");

    let error = sync(&first, &url(&temp.path().join("remote.git"))).unwrap_err();
    assert!(
        error.message.contains("touches more than format"),
        "{}",
        error.message
    );
}

#[test]
fn refuses_a_commit_with_both_format_and_seq_trailers() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second) = pair(temp.path(), FormatVersion(3));

    std::fs::write(second.join(format::FILE), "4\n").unwrap();
    hand_commit(&second, "both\n\nGy-Seq: 2\nGy-Format: 4\n");

    let error = sync(&first, &url(&temp.path().join("remote.git"))).unwrap_err();
    assert!(
        error
            .message
            .contains("has both a Gy-Format and a Gy-Seq trailer"),
        "{}",
        error.message
    );
}
