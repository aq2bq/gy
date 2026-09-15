use gy_ledger::{
    Error, FileStore, FormatVersion, Node, NodeId, NodeKind, Result, Store, format, log,
};
use std::{io::Write, path::Path};

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

fn node_bytes() -> Vec<u8> {
    let node = Node::need(NodeId::from_hash(NodeKind::Need, "0001").unwrap(), "a need").unwrap();
    serde_json::to_vec(&node).unwrap()
}

fn opened(dir: &Path) -> FileStore {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    FileStore::open_with(dir, actor).unwrap()
}

fn commit(store: &mut FileStore, node: &str, why: &str) -> Result<()> {
    store.transaction(|staged| {
        staged.stage(node, node_bytes());
        staged.record(node, "created", why, "conversation");
        Ok(())
    })
}

#[test]
fn a_committed_change_survives_reopen() {
    let temp = tempfile::tempdir().unwrap();
    commit(&mut opened(temp.path()), "n-0001", "the master asked").unwrap();
    let reopened = FileStore::open_with(temp.path(), actor).unwrap();
    assert!(reopened.get("n-0001").is_some());
}

#[test]
fn a_failed_transaction_writes_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = opened(temp.path());
    let result: Result<()> = store.transaction(|staged| {
        staged.stage("n-0001", node_bytes());
        staged.record("n-0001", "created", "why", "source");
        Err(Error::invalid("stop"))
    });
    assert!(result.is_err());
    assert!(
        FileStore::open_with(temp.path(), actor)
            .unwrap()
            .get("n-0001")
            .is_none()
    );
    assert!(log::read(temp.path()).unwrap().0.is_empty());
}

#[test]
fn a_truncated_tail_is_discarded() {
    let temp = tempfile::tempdir().unwrap();
    commit(&mut opened(temp.path()), "n-0001", "why").unwrap();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(temp.path().join(log::FILE))
        .unwrap();
    file.write_all(b"{\"seq\":2,\"at\":1").unwrap();
    drop(file);
    let reopened = FileStore::open_with(temp.path(), actor).unwrap();
    assert!(reopened.get("n-0001").is_some());
    assert_eq!(log::read(temp.path()).unwrap().0.len(), 1);
    let text = std::fs::read_to_string(temp.path().join(log::FILE)).unwrap();
    assert!(text.ends_with('\n') && !text.contains("\"seq\":2"));
}

#[test]
fn history_keeps_when_who_why_and_source() {
    let temp = tempfile::tempdir().unwrap();
    commit(&mut opened(temp.path()), "n-0001", "the master asked").unwrap();
    let reopened = FileStore::open_with(temp.path(), actor).unwrap();
    let entry = &reopened.history()[0];
    assert_eq!(entry.actor.name(), "piko");
    assert_eq!(
        (entry.node.as_str(), entry.what.as_str()),
        ("n-0001", "created")
    );
    assert_eq!(
        (entry.why.as_str(), entry.source.as_str()),
        ("the master asked", "conversation")
    );
    assert!(entry.at > 0);
}
