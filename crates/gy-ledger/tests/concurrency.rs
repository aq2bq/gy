use gy_ledger::{
    FileStore, FormatVersion, Node, NodeId, NodeKind, Result, Store, file, format, log,
};
use std::collections::BTreeSet;
use std::path::Path;

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

fn node_bytes() -> Vec<u8> {
    let node = Node::need(
        NodeId::from_hash(NodeKind::Need, "0001").unwrap(),
        "a",
        "2026-09-15",
        "a need",
    )
    .unwrap();
    serde_json::to_vec(&node).unwrap()
}

fn commit(store: &mut FileStore, node: &str) -> Result<()> {
    store.transaction(|staged| {
        staged.stage(node, node_bytes());
        staged.record(node, "created", "why", "source");
        Ok(())
    })
}

#[test]
fn the_second_writer_is_rejected_until_it_reopens() {
    let temp = tempfile::tempdir().unwrap();
    let dir: &Path = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let mut first = FileStore::open_with(dir, actor).unwrap();
    let mut second = FileStore::open_with(dir, actor).unwrap();
    commit(&mut first, "n-0001").unwrap();
    assert!(commit(&mut second, "n-0002").is_err());
    let mut third = FileStore::open_with(dir, actor).unwrap();
    commit(&mut third, "n-0002").unwrap();
    assert_eq!(log::read(dir).unwrap().0.len(), 2);
}

#[test]
fn commits_from_one_store_are_serialized() {
    let temp = tempfile::tempdir().unwrap();
    let dir: &Path = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let mut store = FileStore::open_with(dir, actor).unwrap();
    commit(&mut store, "n-0001").unwrap();
    commit(&mut store, "n-0002").unwrap();
    let seqs: Vec<u64> = log::read(dir).unwrap().0.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, vec![1, 2]);
}

#[test]
fn an_id_hash_widens_on_collision() {
    let base = 0x1234_5678;
    let mut used = BTreeSet::new();
    assert_eq!(file::unique_hash("n", base, &used).unwrap(), "1234");
    used.insert("n-1234".to_string());
    assert_eq!(file::unique_hash("n", base, &used).unwrap(), "123456");
    used.insert("n-123456".to_string());
    assert_eq!(file::unique_hash("n", base, &used).unwrap(), "12345678");
    used.insert("n-12345678".to_string());
    assert!(file::unique_hash("n", base, &used).is_err());
}
