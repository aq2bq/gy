//! The rebase judge as one rule (n-ef56): a row that creates its own edge
//! target lands, and a row touching a node a refused write created is
//! refused, so no node without a creation lands.
use gy_ledger::{
    Error, FileStore, FormatVersion, Gate, Link, Node, NodeId, NodeKind, Relation, Repository,
    Result, format, log, sync, sync_with,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

/// A gate refusing the creation of ac-0003, so the first row is refused
/// without the remote touching the node (n-557f).
#[derive(Debug)]
struct RefuseAc0003;
impl Gate for RefuseAc0003 {
    fn admit(&self, _before: &BTreeMap<String, Value>, changes: &[log::Change]) -> Result<()> {
        for change in changes {
            if let log::Change::Created { node, .. } = change {
                if node == "ac-0003" {
                    return Err(Error::invalid("the test gate refuses ac-0003"));
                }
            }
        }
        Ok(())
    }
}

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

/// A synthetic row with its writer, like a real write (n-64be): the sync
/// holds writer-less rows before the rebase judge ever sees them.
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

/// A row creating its own edge target lands (ac-b77d): the `req add --need`
/// shape, one row with a created node and an update pointing at it.
#[test]
fn a_row_creating_its_own_target_lands() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
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
            id(NodeKind::Criterion, "0003"),
        )
        .unwrap(),
    );
    log::append(
        &two,
        &event(2, vec![created(&criterion("0003")), created(&need)]),
    )
    .unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert!(report.rejected.is_none());
    assert_eq!(report.rebased, Some(gy_ledger::Range { from: 3, to: 3 }));
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "3");
    assert!(!two.join("rejected.jsonl").exists());
}

/// The ghost stays absent, even rebuilt from the log without the snapshot.
fn assert_no_ghost(dir: &Path) {
    let store = FileStore::open_with(dir, |_| Some("piko".into())).unwrap();
    assert!(
        Repository::new(store)
            .get(&id(NodeKind::Criterion, "0003"))
            .unwrap()
            .is_none()
    );
}

/// An update to a node a refused write created is refused too (ac-ee1f): no
/// node without a creation lands, even rebuilt from the log.
#[test]
fn an_update_to_a_node_a_refused_write_created_is_refused() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    // The remote moves elsewhere, so the rows below go through the rebase
    // without the remote touching ac-0003.
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
    let mut first = criterion("0003");
    first.set_body("the local version");
    log::append(&two, &event(2, vec![created(&first)])).unwrap();
    let mut touched = criterion("0003");
    touched.set_body("the local version again");
    log::append(
        &two,
        &event(
            3,
            vec![log::Change::Updated {
                node: "ac-0003".into(),
                value: serde_json::to_value(touched).unwrap(),
            }],
        ),
    )
    .unwrap();

    let report = sync_with(&two, &url(&remote), Arc::new(RefuseAc0003)).unwrap();
    assert_eq!(report.rejected, Some(2));
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(
        rejected.contains("the test gate refuses ac-0003"),
        "{rejected}"
    );
    assert!(
        rejected.contains("it changes ac-0003, which a refused write created"),
        "{rejected}"
    );

    assert_no_ghost(&two);
    std::fs::remove_file(two.join("snapshot.json")).unwrap();
    assert_no_ghost(&two);
}
