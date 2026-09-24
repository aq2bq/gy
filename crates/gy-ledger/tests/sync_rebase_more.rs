//! The rest of the rebase guarantees (n-ecbf 2B): an id clash, a refused
//! chain, the push rollback, the snapshot, and `rejected.jsonl`.
use gy_ledger::{
    FileStore, Filter, FormatVersion, Link, Listing, Node, NodeId, NodeKind, Relation, Repository,
    format, list, log, sync,
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

fn deleted(node: &Node) -> log::Change {
    log::Change::Deleted {
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

fn store(dir: &Path) -> FileStore {
    FileStore::open_with(dir, |_| Some("piko".into())).unwrap()
}

#[test]
fn a_duplicate_id_is_refused() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    // The remote moves without touching ac-0001; the copy creates it again.
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
    log::append(&two, &event(2, vec![created(&criterion("0001"))])).unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1));
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(
        rejected.contains("already exists in the remote's ledger"),
        "{rejected}"
    );
}

#[test]
fn a_write_on_a_refused_write_is_refused_in_chain() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![deleted(&criterion("0001"))]);

    let mut need = Node::need(
        id(NodeKind::Need, "0003"),
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
    log::append(&two, &event(2, vec![created(&criterion("0001"))])).unwrap();
    log::append(&two, &event(3, vec![created(&need)])).unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(2));
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(
        rejected.contains("the remote deleted ac-0001"),
        "{rejected}"
    );
    assert!(
        rejected.contains("which a refused write created"),
        "{rejected}"
    );
    assert_eq!(rejected.lines().count(), 2);
}

#[test]
fn a_failed_push_leaves_the_copy_behind_the_remote() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
    log::append(&two, &event(2, vec![created(&criterion("0003"))])).unwrap();

    let objects = remote.join("objects");
    let original = std::fs::metadata(&objects).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    std::fs::set_permissions(&objects, readonly).unwrap();
    let error = sync(&two, &url(&remote)).unwrap_err();
    std::fs::set_permissions(&objects, original).unwrap();
    assert!(
        error.message.contains("refused the push"),
        "{}",
        error.message
    );

    // The commit is rolled back; the write is still in the working log (now
    // renumbered after the remote's line).
    assert_eq!(
        git(&two, &["rev-parse", "HEAD"]).trim(),
        git(&two, &["rev-parse", "origin/main"]).trim()
    );
    assert_eq!(last_seq(&two), 3);

    let report = sync(&two, &url(&remote)).unwrap();
    assert!(report.pushed.is_some());
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "3");
    assert_eq!(last_seq(&two), 3);
}

#[test]
fn the_snapshot_and_list_agree_after_a_rebase() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
    log::append(&two, &event(2, vec![created(&criterion("0003"))])).unwrap();
    sync(&two, &url(&remote)).unwrap();

    let repository = Repository::new(store(&two));
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
    match list(&repository, &Filter::default()).unwrap() {
        Listing::Nodes(rows) => assert_eq!(rows.len(), 3),
        Listing::History(_) => panic!("expected nodes"),
    }
    let snapshot = std::fs::read_to_string(two.join("snapshot.json")).unwrap();
    assert!(
        snapshot.contains(&format!("\"seq\":{}", last_seq(&two))),
        "{snapshot}"
    );
}

#[test]
fn rejected_jsonl_names_the_peer_and_the_write() {
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
            value: serde_json::to_value(&changed).unwrap(),
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
    sync(&two, &url(&remote)).unwrap();

    let text = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    let rejected: serde_json::Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    assert!(
        rejected["reason"]
            .as_str()
            .unwrap()
            .contains("changed ac-0001")
    );
    assert_eq!(rejected["event"]["seq"], 2);
    assert_eq!(rejected["by"]["seq"], 2);
    assert_eq!(rejected["by"]["actor"], "piko");
}
