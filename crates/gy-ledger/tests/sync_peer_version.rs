//! A peer writing with a newer gy (n-670a B, r-3f07): the pulled
//! `Gy-Version` trailers land in the sync report and in `sync.state`, and a
//! commit carrying one still passes the guard.
use gy_ledger::{FormatVersion, Node, NodeId, NodeKind, format, log, sync};
use std::path::{Path, PathBuf};
use std::process::Command;

mod common;
use common::ident;

fn git(cwd: &Path, args: &[&str]) {
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
}

fn url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        id(NodeKind::Criterion, hash),
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
        why: "test".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: Some("piko".to_string()),
        by_mail: Some("piko@example.com".to_string()),
        changes,
    }
}

fn created(node: &Node) -> log::Change {
    log::Change::Created {
        node: node.id().to_string(),
        value: serde_json::to_value(node).unwrap(),
    }
}

fn ledger(dir: &Path, events: &[log::Event]) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    for event in events {
        log::append(dir, event).unwrap();
    }
}

/// A peer commit whose message carries `Gy-Version`, if given.
fn peer_commit(dir: &Path, seq: u64, changes: Vec<log::Change>, version: Option<&str>) {
    log::append(dir, &event(seq, changes)).unwrap();
    let mut message = "peer\n\nactor: piko\n".to_string();
    if let Some(version) = version {
        message.push_str(&format!("Gy-Version: {version}\n"));
    }
    message.push_str(&format!("Gy-Seq: {seq}\n"));
    git(dir, &["add", log::FILE]);
    let out = Command::new("git")
        .args(["commit", "-q", "-m", &message])
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    git(dir, &["push", "-q", "origin", "HEAD:main"]);
}

fn pair(temp: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let one = temp.join("one");
    let remote = temp.join("remote.git");
    ledger(&one, &[event(1, vec![created(&criterion("0001"))])]);
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    let two = temp.join("two");
    sync(&two, &url(&remote)).unwrap();
    (one, two, remote)
}

/// A pulled `Gy-Version` lands in the report and in `sync.state`, and the
/// commit carrying it passes the guard.
#[test]
fn pulled_peer_version_is_reported_and_kept() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![created(&criterion("0002"))], Some("9.9.9"));

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.peer_version.as_deref(), Some("9.9.9"));
    let state = std::fs::read_to_string(two.join("sync.state")).unwrap();
    assert!(state.contains("9.9.9"), "{state}");
}

/// No version on the wire means no peer version anywhere.
#[test]
fn matching_versions_report_nothing() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![created(&criterion("0002"))], None);

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.peer_version, None);
    let state = std::fs::read_to_string(two.join("sync.state")).unwrap();
    assert!(!state.contains("peer_version"), "{state}");
}
