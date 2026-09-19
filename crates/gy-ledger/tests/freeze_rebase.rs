//! n-a3f2 (4): the rebase entry runs the same rule. An unpushed write that
//! changes an approved requirement's body — the shape an older build could
//! leave behind — is refused when the rebase carries it onto the remote. The
//! local write goes through the open gate to stand in for that older build;
//! the remote copy judges it with the rule. The two-copy harness mirrors
//! `sync_rebase.rs`.
use gy_ledger::{
    Edit, FileStore, FormatVersion, Link, Node, NodeId, NodeKind, Open, Operation, Relation,
    Repository, RequirementState, format, log, sync,
};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn ident() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        for (key, value) in [
            ("GIT_AUTHOR_NAME", "piko"),
            ("GIT_AUTHOR_EMAIL", "piko@example.com"),
            ("GIT_COMMITTER_NAME", "piko"),
            ("GIT_COMMITTER_EMAIL", "piko@example.com"),
        ] {
            std::env::set_var(key, value);
        }
    });
}

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

fn criterion(hash: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "an ac").unwrap()
}

/// The approved requirement the frozen rule guards, targeting the first
/// criterion.
fn approved() -> Node {
    let ac = criterion("0001");
    let mut node = Node::requirement(
        id(NodeKind::Requirement, "0002"),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Approved,
    )
    .unwrap();
    node.link(Link::new(node.id().clone(), Relation::Targets, ac.id().clone()).unwrap());
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

#[test]
fn a_rebase_refuses_a_write_that_changed_an_approved_requirement() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let one = temp.path().join("one");
    let remote = temp.path().join("remote.git");
    let two = temp.path().join("two");
    let request = approved();

    ledger(
        &one,
        &[event(
            1,
            vec![created(&criterion("0001")), created(&request)],
        )],
    );
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    sync(&two, &url(&remote)).unwrap();

    // The peer advances the remote, so the copy has to rebase its write.
    peer_commit(&one, 2, vec![created(&criterion("0003"))]);

    // An older build's write: the approved requirement's body changes, let
    // through here by the open gate.
    let store = FileStore::open_with(&two, |_| Some("piko".into())).unwrap();
    let mut repo = Repository::with_gate(store, Arc::new(Open));
    Edit {
        id: request.id().clone(),
        reason: "an older build".into(),
        title: None,
        body: Some("changed after approval".into()),
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1));
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("is approved"), "{rejected}");
    assert!(rejected.contains("req revise"), "{rejected}");
}
