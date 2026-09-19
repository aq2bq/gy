//! n-557f: the gate at the rebase entry. A refused candidate goes to
//! `rejected.jsonl` with its reason; the other candidates still land.
use gy_ledger::{
    Error, FileStore, FormatVersion, Gate, Node, NodeId, NodeKind, Repository, Result, format, log,
    sync, sync_with,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

/// A gate that refuses the write creating the named node.
#[derive(Debug)]
struct RefuseNode(&'static str);
impl Gate for RefuseNode {
    fn admit(&self, _before: &BTreeMap<String, Value>, changes: &[log::Change]) -> Result<()> {
        for change in changes {
            if let log::Change::Created { node, .. } = change {
                if node == self.0 {
                    return Err(Error::invalid("the test gate refuses this write"));
                }
            }
        }
        Ok(())
    }
}

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
fn a_refused_candidate_is_recorded_and_the_others_land() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
    log::append(&two, &event(2, vec![created(&criterion("0003"))])).unwrap();
    log::append(&two, &event(3, vec![created(&criterion("0004"))])).unwrap();

    let report = sync_with(&two, &url(&remote), Arc::new(RefuseNode("ac-0003"))).unwrap();
    assert_eq!(report.rejected, Some(1));
    assert_eq!(report.rebased, Some(gy_ledger::Range { from: 3, to: 3 }));

    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(
        rejected.contains("the test gate refuses this write"),
        "{rejected}"
    );

    let store = FileStore::open_with(&two, |_| Some("piko".into())).unwrap();
    let repository = Repository::new(store);
    let landed = repository.get(&id(NodeKind::Criterion, "0004")).unwrap();
    assert!(landed.is_some());
    let refused = repository.get(&id(NodeKind::Criterion, "0003")).unwrap();
    assert!(refused.is_none());
}
