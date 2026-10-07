//! n-d5a2, d-ee34: when a rebase would file a requirement onto a need the
//! ledger already closed, the gate refuses that line and the rest lands.
use gy_ledger::{
    Closed, ClosedBy, FileStore, FormatVersion, Link as ModelLink, Node, NodeData, NodeId,
    NodeKind, Relation, Repository, RequirementState, format, log, sync,
};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::ident;

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
fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}
fn need(hash: &str) -> Node {
    Node::need(
        id(NodeKind::Need, hash),
        "a",
        "2026-09-15T00:00:00Z",
        "a need",
    )
    .unwrap()
}
/// A need the ledger already holds closed (the shape of an older copy).
fn closed_need(hash: &str) -> Node {
    let mut node = need(hash);
    if let NodeData::Need(data) = node.data_mut() {
        data.closed = Some(Closed {
            by: ClosedBy::Fact,
            evidence: "done".to_string(),
        });
    }
    node
}
fn requirement(hash: &str) -> Node {
    Node::requirement(
        id(NodeKind::Requirement, hash),
        "a",
        "2026-09-15T00:00:00Z",
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap()
}
/// The closed need with one filed-as edge, for an updated row.
fn pointing(hash: &str, to: &Node) -> Node {
    let mut node = closed_need(hash);
    node.link(ModelLink::new(node.id().clone(), Relation::FiledAs, to.id().clone()).unwrap());
    node
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
fn updated(node: &Node) -> log::Change {
    log::Change::Updated {
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
fn peer_commit(dir: &Path, seq: u64, changes: Vec<log::Change>) {
    log::append(dir, &event(seq, changes)).unwrap();
    git(dir, &["add", log::FILE]);
    git(
        dir,
        &[
            "commit",
            "-q",
            "-m",
            &format!("peer\n\nactor: piko\nGy-Seq: {seq}"),
        ],
    );
    git(dir, &["push", "-q", "origin", "HEAD:main"]);
}

/// The clone and its remote, plus the peer that pushes to it.
fn pair(temp: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let one = temp.join("one");
    let remote = temp.join("remote.git");
    ledger(
        &one,
        &[event(
            1,
            vec![created(&closed_need("0001")), created(&requirement("0002"))],
        )],
    );
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    let two = temp.join("two");
    sync(&two, &url(&remote)).unwrap();
    (one, two, remote)
}

#[test]
fn a_rebase_that_would_file_a_requirement_on_a_closed_need_refuses_that_line() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());

    // The remote grows by an unrelated node, so the local line must rebase. The
    // local pending line (the shape of an older gy, before the gate) adds the
    // filed-as edge to the closed need.
    peer_commit(&one, 2, vec![created(&need("0003"))]);
    log::append(
        &two,
        &event(2, vec![updated(&pointing("0001", &requirement("0002")))]),
    )
    .unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1), "{report:?}");

    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("is closed"), "{rejected}");

    // The need stays closed with no filed-as edge.
    let store = FileStore::open_with(&two, |_| Some("piko".into())).unwrap();
    let repository = Repository::new(store);
    let n = repository
        .get(&id(NodeKind::Need, "0001"))
        .unwrap()
        .unwrap();
    assert!(
        matches!(n.data(), NodeData::Need(data) if data.closed.is_some()),
        "{n:?}"
    );
    assert!(
        n.links().iter().all(|edge| edge.label != Relation::FiledAs),
        "{n:?}"
    );
}
