//! n-e299, d-858d: when a rebase would put a local unsent line and a remote
//! line together into a depends-on cycle, the gate refuses that line and the
//! remote's line still lands.
use gy_ledger::{
    FileStore, FormatVersion, Link as ModelLink, Node, NodeId, NodeKind, Relation, Repository,
    format, log, sync,
};
use std::path::{Path, PathBuf};
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
/// The same need with one depends-on edge, for an updated row.
fn pointing(hash: &str, to: &Node) -> Node {
    let mut node = need(hash);
    node.link(ModelLink::new(node.id().clone(), Relation::DependsOn, to.id().clone()).unwrap());
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

/// Two copies of one ledger that holds n-0001 and n-0002.
fn pair(temp: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let one = temp.join("one");
    let remote = temp.join("remote.git");
    ledger(
        &one,
        &[event(
            1,
            vec![created(&need("0001")), created(&need("0002"))],
        )],
    );
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    let two = temp.join("two");
    sync(&two, &url(&remote)).unwrap();
    (one, two, remote)
}

#[test]
fn a_rebase_that_would_close_a_cycle_refuses_that_line() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());

    // The remote adds n2 -> n1; the local unsent line adds n1 -> n2.
    peer_commit(&one, 2, vec![updated(&pointing("0002", &need("0001")))]);
    log::append(
        &two,
        &event(2, vec![updated(&pointing("0001", &need("0002")))]),
    )
    .unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1), "{report:?}");

    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("would form a cycle"), "{rejected}");

    // The remote's edge landed; the local one did not.
    let store = FileStore::open_with(&two, |_| Some("piko".into())).unwrap();
    let repository = Repository::new(store);
    let n1 = repository
        .get(&id(NodeKind::Need, "0001"))
        .unwrap()
        .unwrap();
    assert!(
        n1.links()
            .iter()
            .all(|edge| edge.label != Relation::DependsOn),
        "{n1:?}"
    );
}
