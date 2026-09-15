use gy_ledger::{Actor, FileStore, FormatVersion, MemoryStore, NodeId, NodeKind, format};
use std::path::Path;

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

#[test]
fn two_processes_in_the_same_second_mint_distinct_short_ids() {
    let temp = tempfile::tempdir().unwrap();
    let dir: &Path = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let mut first = FileStore::open_with(dir, actor).unwrap();
    let mut second = FileStore::open_with(dir, actor).unwrap();
    let a = NodeId::mint(NodeKind::Need, &mut first).unwrap();
    let b = NodeId::mint(NodeKind::Need, &mut second).unwrap();
    assert_eq!(a.hash().len(), 4, "{a}");
    assert_eq!(b.hash().len(), 4, "{b}");
    assert_ne!(a, b);
}

#[test]
fn the_memory_store_mints_the_same_way() {
    let mut first = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut second = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let a = NodeId::mint(NodeKind::Need, &mut first).unwrap();
    let b = NodeId::mint(NodeKind::Need, &mut second).unwrap();
    assert_eq!(a.hash().len(), 4, "{a}");
    assert_eq!(b.hash().len(), 4, "{b}");
    assert_ne!(a, b);
}
