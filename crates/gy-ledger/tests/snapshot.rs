use gy_ledger::{FileStore, FormatVersion, Node, NodeId, NodeKind, Store, format};
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

fn put(store: &mut FileStore, key: &str, hash: &str, title: &str) {
    store
        .transaction(|staged| {
            staged.stage(key, node(hash, title));
            staged.record(key, "created", "asked", "conversation");
            Ok(())
        })
        .unwrap();
}

fn snapshot(dir: &Path) -> std::path::PathBuf {
    dir.join("snapshot.json")
}

#[test]
fn a_commit_writes_a_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    put(&mut opened(temp.path()), "n-0001", "0001", "one");
    assert!(snapshot(temp.path()).is_file());
}

#[test]
fn deleting_the_snapshot_changes_nothing() {
    let temp = tempfile::tempdir().unwrap();
    put(&mut opened(temp.path()), "n-0001", "0001", "one");
    std::fs::remove_file(snapshot(temp.path())).unwrap();
    let reopened = FileStore::open_with(temp.path(), actor).unwrap();
    assert!(reopened.get("n-0001").is_some());
    assert!(snapshot(temp.path()).is_file());
}

#[test]
fn a_corrupt_snapshot_is_ignored() {
    let temp = tempfile::tempdir().unwrap();
    put(&mut opened(temp.path()), "n-0001", "0001", "one");
    std::fs::write(snapshot(temp.path()), "not json").unwrap();
    let reopened = FileStore::open_with(temp.path(), actor).unwrap();
    assert!(reopened.get("n-0001").is_some());
    assert_eq!(reopened.replayed(), 1);
}

#[test]
fn only_the_rows_after_the_snapshot_are_replayed() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    put(&mut store, "n-0001", "0001", "one");
    let first = std::fs::read(snapshot(temp.path())).unwrap();
    put(&mut store, "n-0002", "0002", "two");
    std::fs::write(snapshot(temp.path()), first).unwrap();
    let reopened = FileStore::open_with(temp.path(), actor).unwrap();
    assert!(reopened.get("n-0001").is_some() && reopened.get("n-0002").is_some());
    assert_eq!(reopened.replayed(), 1);
}
