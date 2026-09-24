use gy_ledger::{
    CriterionSatisfy, FileStore, FormatVersion, Link, Node, NodeId, NodeKind, Operation, Relation,
    Repository, RequirementState, Rules, format, log, share_upload, sync,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
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
        // Well-formed like a real write: this file is about the rebase,
        // and writer-less rows no longer reach a remote (n-64be).
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
fn last_seq(dir: &Path) -> u64 {
    log::read(dir).unwrap().0.last().unwrap().seq
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
#[test]
fn different_writes_both_land_and_are_renumbered() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
    log::append(&two, &event(2, vec![created(&criterion("0003"))])).unwrap();
    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.pulled.as_ref().unwrap().to, 2);
    assert_eq!(report.rebased, Some(gy_ledger::Range { from: 3, to: 3 }));
    assert!(report.rejected.is_none());
    assert_eq!(last_seq(&two), 3);
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "3");
    let store = FileStore::open_with(&two, |_| Some("piko".into())).unwrap();
    let repository = Repository::new(store);
    assert!(
        repository
            .get(&id(NodeKind::Criterion, "0002"))
            .unwrap()
            .is_some()
    );
    assert!(
        repository
            .get(&id(NodeKind::Criterion, "0003"))
            .unwrap()
            .is_some()
    );
}
#[test]
fn a_write_on_a_node_the_remote_changed_is_refused() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    let mut changed = criterion("0001");
    changed.set_body("the remote's version");
    peer_commit(
        &one,
        2,
        vec![log::Change::Updated {
            node: changed.id().to_string(),
            value: serde_json::to_value(changed).unwrap(),
        }],
    );
    log::append(
        &two,
        &event(
            2,
            vec![log::Change::Updated {
                node: "ac-0001".into(),
                value: serde_json::to_value(criterion("0001")).unwrap(),
            }],
        ),
    )
    .unwrap();
    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1));
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "2");
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(
        rejected.contains("the remote changed ac-0001"),
        "{rejected}"
    );
}
#[test]
fn an_edge_to_a_deleted_node_is_refused() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(
        &one,
        2,
        vec![log::Change::Deleted {
            node: "ac-0001".into(),
            value: serde_json::to_value(criterion("0001")).unwrap(),
        }],
    );
    let mut need = Node::need(
        id(NodeKind::Need, "0002"),
        "a",
        "2026-09-15T00:00:00Z",
        "a need",
    )
    .unwrap();
    need.link(
        Link::new(
            need.id().clone(),
            Relation::Targets,
            id(NodeKind::Criterion, "0001"),
        )
        .unwrap(),
    );
    log::append(&two, &event(2, vec![created(&need)])).unwrap();
    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1));
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("points at ac-0001"), "{rejected}");
}

/// Two copies whose approved requirement covering ac-0001 the peer sends back
/// to filed; the local copy records a satisfy that no longer has coverage
/// (n-f921).
fn revised_away(temp: &Path) -> (PathBuf, PathBuf) {
    let one = temp.join("one");
    let remote = temp.join("remote.git");
    let ac = criterion("0001");
    let need = Node::need(
        id(NodeKind::Need, "0002"),
        "a",
        "2026-09-15T00:00:00Z",
        "a need",
    )
    .unwrap();
    let mut approved = Node::requirement(
        id(NodeKind::Requirement, "0003"),
        "a",
        "2026-09-15T00:00:00Z",
        "a requirement",
        RequirementState::Approved,
    )
    .unwrap();
    approved.link(Link::new(approved.id().clone(), Relation::Targets, ac.id().clone()).unwrap());
    ledger(
        &one,
        &[event(
            1,
            vec![created(&need), created(&ac), created(&approved)],
        )],
    );
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    let two = temp.join("two");
    sync(&two, &url(&remote)).unwrap();

    // The peer sends the same requirement back to filed and pushes it...
    approved.advance(RequirementState::Filed).unwrap();
    peer_commit(
        &one,
        2,
        vec![log::Change::Updated {
            node: approved.id().to_string(),
            value: serde_json::to_value(&approved).unwrap(),
        }],
    );

    // ...while the local copy records the satisfy it still saw covered.
    let mut repo = Repository::new(FileStore::open_with(&two, |_| Some("piko".into())).unwrap());
    CriterionSatisfy {
        id: id(NodeKind::Criterion, "0001"),
        evidence: "x".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    (two, remote)
}

#[test]
fn a_satisfy_the_peer_revised_away_is_refused_on_rebase() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (two, remote) = revised_away(temp.path());
    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1));
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("ac-0001"), "{rejected}");
}

#[test]
fn share_judges_its_first_upload_with_the_rule() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (two, remote) = revised_away(temp.path());
    share_upload(&two, &url(&remote), "main", Arc::new(Rules)).unwrap();
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("ac-0001"), "{rejected}");
}
