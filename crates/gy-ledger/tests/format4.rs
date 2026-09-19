//! The 3 → 4 format raise (n-96f8, ac-682e): opening a version 3 ledger keeps
//! `format.3.bak` and leaves the log byte for byte, with or without a snapshot.
use gy_ledger::{FileStore, FormatVersion, Node, NodeId, NodeKind, Repository, Store, format, log};
use std::path::Path;

const DATE: &str = "2026-09-15T00:00:00Z";

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

fn event(seq: u64, changes: Vec<log::Change>) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: "piko".to_string(),
        why: "seed".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: None,
        by_mail: None,
        changes,
    }
}

fn need(hash: &str) -> Node {
    Node::need(
        NodeId::from_hash(NodeKind::Need, hash).unwrap(),
        "a",
        DATE,
        "a need",
    )
    .unwrap()
}

fn created(value: &Node) -> log::Change {
    log::Change::Created {
        node: value.id().to_string(),
        value: serde_json::to_value(value).unwrap(),
    }
}

/// A version 3 ledger with one need, optionally carrying a snapshot.
fn version_three(dir: &Path, snapshot: bool) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion(3)).unwrap();
    let node = need("0001");
    log::append(dir, &event(1, vec![created(&node)])).unwrap();
    if snapshot {
        let text = serde_json::json!({
            "seq": 1,
            "nodes": {"n-0001": serde_json::to_value(&node).unwrap()},
        });
        std::fs::write(dir.join("snapshot.json"), text.to_string()).unwrap();
    }
}

#[test]
fn opening_a_version_three_ledger_raises_the_format_and_keeps_a_backup() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("ledger");
    version_three(&dir, false);
    let events = std::fs::read(dir.join(log::FILE)).unwrap();

    let first = FileStore::open_with(&dir, actor).unwrap();
    assert_eq!(first.migrated(), Some((3, 4)));
    assert_eq!(first.version(), FormatVersion::CURRENT);
    assert_eq!(format::read(&dir).unwrap(), FormatVersion::CURRENT);
    assert!(dir.join("format.3.bak").is_file());
    assert_eq!(std::fs::read(dir.join(log::FILE)).unwrap(), events);

    // A second open has nothing left to do.
    let second = FileStore::open_with(&dir, actor).unwrap();
    assert_eq!(second.migrated(), None);
    assert_eq!(std::fs::read(dir.join(log::FILE)).unwrap(), events);
}

#[test]
fn the_raise_is_the_same_with_and_without_a_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    for snapshot in [false, true] {
        let dir = temp.path().join(if snapshot { "with" } else { "without" });
        version_three(&dir, snapshot);
        let events = std::fs::read(dir.join(log::FILE)).unwrap();

        let store = FileStore::open_with(&dir, actor).unwrap();
        assert_eq!(store.migrated(), Some((3, 4)));
        assert!(dir.join("format.3.bak").is_file());
        assert_eq!(format::read(&dir).unwrap(), FormatVersion::CURRENT);
        assert_eq!(std::fs::read(dir.join(log::FILE)).unwrap(), events);

        let repository = Repository::new(store);
        let id = NodeId::from_hash(NodeKind::Need, "0001").unwrap();
        assert_eq!(repository.get(&id).unwrap().unwrap().title(), "a need");
    }
}

#[test]
fn a_version_five_ledger_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join(format::FILE), "5\n").unwrap();
    let error = FileStore::open_with(temp.path(), actor).err().unwrap();
    assert!(
        error.message.contains("newer than this build supports"),
        "{}",
        error.message
    );
}
