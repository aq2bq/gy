use gy_ledger::{FileStore, FormatVersion, Node, NodeId, NodeKind, Result, Store, format, log};
use std::path::Path;

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

fn node(hash: &str, title: &str) -> Vec<u8> {
    let node = Node::need(
        NodeId::from_hash(NodeKind::Need, hash).unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        title,
    )
    .unwrap();
    serde_json::to_vec(&node).unwrap()
}

fn opened(dir: &Path) -> FileStore {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    FileStore::open_with(dir, actor).unwrap()
}

fn put(store: &mut FileStore, key: &str, hash: &str, title: &str, what: &str) {
    store
        .transaction(|staged| {
            staged.stage(key, node(hash, title));
            staged.record(key, what, "asked", "conversation");
            Ok(())
        })
        .unwrap();
}

fn delete(store: &mut FileStore, key: &str) {
    store
        .transaction(|staged| {
            staged.stage(key, Vec::new());
            staged.record(key, "deleted", "not needed", "conversation");
            Ok(())
        })
        .unwrap();
}

#[test]
fn undoing_a_creation_removes_the_node() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    put(&mut store, "n-0001", "0001", "one", "created");
    assert!(store.get("n-0001").is_some());
    store.undo("mistake", "conversation").unwrap();
    assert!(store.get("n-0001").is_none());
    assert!(
        FileStore::open_with(temp.path(), actor)
            .unwrap()
            .get("n-0001")
            .is_none()
    );
}

#[test]
fn undoing_an_update_restores_the_earlier_value() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    put(&mut store, "n-0001", "0001", "one", "created");
    put(&mut store, "n-0001", "0001", "two", "updated");
    assert!(text(store.get("n-0001")).contains("two"));
    store.undo("revert", "conversation").unwrap();
    let now = text(store.get("n-0001"));
    assert!(now.contains("one") && !now.contains("two"), "{now}");
}

#[test]
fn undoing_a_deletion_brings_the_node_back() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    put(&mut store, "n-0001", "0001", "one", "created");
    delete(&mut store, "n-0001");
    assert!(store.get("n-0001").is_none());
    store.undo("restore", "conversation").unwrap();
    assert!(text(store.get("n-0001")).contains("one"));
}

#[test]
fn undoing_the_undo_restores_the_creation() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    put(&mut store, "n-0001", "0001", "one", "created");
    store.undo("mistake", "conversation").unwrap();
    assert!(store.get("n-0001").is_none());
    store.undo("undo the mistake", "conversation").unwrap();
    assert!(store.get("n-0001").is_some());
}

#[test]
fn the_undo_line_keeps_why_and_source() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    put(&mut store, "n-0001", "0001", "one", "created");
    store.undo("mistake", "conversation").unwrap();
    let reopened = FileStore::open_with(temp.path(), actor).unwrap();
    let line = reopened.history().last().unwrap();
    assert_eq!(line.actor.name(), "piko");
    assert_eq!(
        (line.why.as_str(), line.source.as_str()),
        ("mistake", "conversation")
    );
}

#[test]
fn a_failed_commit_rolls_back_the_staged_changes() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let mut first = FileStore::open_with(dir, actor).unwrap();
    let mut second = FileStore::open_with(dir, actor).unwrap();
    put(&mut first, "n-0001", "0001", "one", "created");
    let result: Result<()> = second.transaction(|staged| {
        staged.stage("n-0002", node("0002", "two"));
        staged.record("n-0002", "created", "why", "source");
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(log::read(dir).unwrap().0.len(), 1);
    // A new transaction on the same store does not carry the failed change.
    let again: Result<()> = second.transaction(|staged| {
        staged.stage("n-0003", node("0003", "three"));
        staged.record("n-0003", "created", "why", "source");
        Ok(())
    });
    assert!(again.is_err());
    assert_eq!(log::read(dir).unwrap().0.len(), 1);
}

#[test]
fn a_record_only_commit_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    let result: Result<()> = store.transaction(|staged| {
        staged.record("n-0001", "created", "why", "source");
        Ok(())
    });
    assert!(result.is_err());
    assert!(log::read(temp.path()).unwrap().0.is_empty());
}

fn text(value: Option<Vec<u8>>) -> String {
    String::from_utf8(value.expect("a node")).unwrap()
}
