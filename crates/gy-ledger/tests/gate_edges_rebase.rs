//! d-fb49: the gate refuses a rebase line that would add a duplicate edge or a
//! closes edge no decision made (a rebase line does not pass `Node::link`).
use gy_ledger::{
    Closure, DecisionScope, FileStore, FormatVersion, Link as ModelLink, Node, NodeData, NodeId,
    NodeKind, Relation, Repository, format, log, sync,
};
use std::path::{Path, PathBuf};
use std::process::Command;

mod common;
use common::ident;

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

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
    Node::need(id(NodeKind::Need, hash), SCOPE, DATE, "a need").unwrap()
}
fn criterion(hash: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "an ac").unwrap()
}
fn question(hash: &str) -> Node {
    Node::question(id(NodeKind::Question, hash), SCOPE, DATE, "a question").unwrap()
}
fn decision(hash: &str) -> Node {
    let scope = DecisionScope::recorded("scope").unwrap();
    Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        "a decision",
        scope,
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
fn change(created: bool, node: &Node) -> log::Change {
    let key = node.id().to_string();
    let value = serde_json::to_value(node).unwrap();
    if created {
        log::Change::Created { node: key, value }
    } else {
        log::Change::Updated { node: key, value }
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
    let message = format!("peer\n\nactor: piko\nGy-Seq: {seq}");
    git(dir, &["commit", "-q", "-m", &message]);
    git(dir, &["push", "-q", "origin", "HEAD:main"]);
}
/// `one` and its remote, branched from `base`, plus the peer that pushes.
fn pair(temp: &Path, base: Vec<log::Change>) -> (PathBuf, PathBuf, PathBuf) {
    let one = temp.join("one");
    let remote = temp.join("remote.git");
    ledger(&one, &[event(1, base)]);
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    let two = temp.join("two");
    sync(&two, &url(&remote)).unwrap();
    (one, two, remote)
}
fn node(dir: &Path, kind: NodeKind, hash: &str) -> Node {
    let store = FileStore::open_with(dir, |_| Some("piko".into())).unwrap();
    Repository::new(store)
        .get(&id(kind, hash))
        .unwrap()
        .unwrap()
}
/// The need with two copies of the same targets edge, as an older gy wrote it.
fn doubled(hash: &str, to: &Node) -> Node {
    let mut node = need(hash);
    let edge = ModelLink::new(node.id().clone(), Relation::Targets, to.id().clone()).unwrap();
    node.link(edge);
    let mut value = serde_json::to_value(&node).unwrap();
    let edges = value.get_mut("links").unwrap().as_array_mut().unwrap();
    edges.push(edges[0].clone());
    serde_json::from_value(value).unwrap()
}
/// The question closed by a fact yet pointing at a decision with a closes edge.
fn fact_closed(hash: &str, to: &Node) -> Node {
    let mut node = question(hash);
    if let NodeData::Question(data) = node.data_mut() {
        data.closure = Some(Closure::Fact);
        data.evidence = Some("an older copy".to_string());
    }
    let edge = ModelLink::new(node.id().clone(), Relation::Closes, to.id().clone()).unwrap();
    node.link(edge);
    node
}
#[test]
fn a_rebase_that_adds_a_duplicate_edge_refuses_that_line() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(
        temp.path(),
        vec![
            change(true, &need("0001")),
            change(true, &criterion("0002")),
        ],
    );
    peer_commit(&one, 2, vec![change(true, &criterion("0003"))]);
    let offending = doubled("0001", &criterion("0002"));
    log::append(&two, &event(2, vec![change(false, &offending)])).unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1), "{report:?}");
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("is already registered"), "{rejected}");

    // The need keeps its base shape: no targets edge landed.
    let n = node(&two, NodeKind::Need, "0001");
    assert!(
        n.links()
            .iter()
            .all(|edge| edge.reversed || edge.label != Relation::Targets),
        "{n:?}"
    );
}
#[test]
fn a_rebase_that_adds_a_non_decision_closes_refuses_that_line() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(
        temp.path(),
        vec![
            change(true, &question("0001")),
            change(true, &decision("0002")),
        ],
    );
    peer_commit(&one, 2, vec![change(true, &need("0003"))]);
    let offending = fact_closed("0001", &decision("0002"));
    log::append(&two, &event(2, vec![change(false, &offending)])).unwrap();

    let report = sync(&two, &url(&remote)).unwrap();
    assert_eq!(report.rejected, Some(1), "{report:?}");
    let rejected = std::fs::read_to_string(two.join("rejected.jsonl")).unwrap();
    assert!(rejected.contains("closed by decision"), "{rejected}");

    // The question stays open with no closes edge.
    let q = node(&two, NodeKind::Question, "0001");
    assert!(
        matches!(q.data(), NodeData::Question(data) if data.closure.is_none()),
        "{q:?}"
    );
    assert!(
        q.links()
            .iter()
            .all(|edge| edge.reversed || edge.label != Relation::Closes),
        "{q:?}"
    );
}
