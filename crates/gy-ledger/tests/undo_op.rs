use gy_ledger::{
    Actor, Edit, FileStore, FormatVersion, MemoryStore, NeedAdd, Node, NodeId, NodeKind, Operation,
    Repository, Store, Undo, format,
};
use std::path::Path;

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn memory() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn file(dir: &Path) -> Repository<FileStore> {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let store = FileStore::open_with(dir, |_| Some("piko".into())).unwrap();
    Repository::new(store)
}

fn add_need<S: Store>(repo: &mut Repository<S>) -> NodeId {
    let criterion = Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, "0001").unwrap(),
        SCOPE,
        DATE,
        "a criterion",
    )
    .unwrap();
    let target = criterion.id().clone();
    repo.transaction("seed", "test", |repo| repo.put(&criterion))
        .unwrap();
    NeedAdd {
        scope: SCOPE.into(),
        title: "a need".into(),
        targets: vec![target],
        spawned_by: None,
        body: None,
    }
    .run(repo)
    .unwrap()
    .value
}

fn retitle(id: &NodeId, title: &str) -> Edit {
    Edit {
        id: id.clone(),
        reason: "clarify".into(),
        title: Some(title.into()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
}

fn undo(reason: &str) -> Undo {
    Undo {
        reason: reason.into(),
    }
}

#[test]
fn undo_removes_a_created_need() {
    let mut repo = memory();
    let id = add_need(&mut repo);
    assert!(repo.get(&id).unwrap().is_some());
    let outcome = undo("mistake").run(&mut repo).unwrap();
    assert_eq!(outcome.changed, ["undone"]);
    assert!(repo.get(&id).unwrap().is_none());
}

#[test]
fn undo_restores_an_edited_value() {
    let mut repo = memory();
    let id = add_need(&mut repo);
    retitle(&id, "new").run(&mut repo).unwrap();
    assert_eq!(repo.get(&id).unwrap().unwrap().title(), "new");
    undo("revert").run(&mut repo).unwrap();
    assert_eq!(repo.get(&id).unwrap().unwrap().title(), "a need");
}

#[test]
fn undo_reports_when_there_is_nothing_to_undo_or_no_reason() {
    let mut repo = memory();
    assert!(undo("mistake").run(&mut repo).is_err());
    add_need(&mut repo);
    assert!(undo("  ").run(&mut repo).is_err());
}

#[test]
fn the_file_store_undoes_the_same_way() {
    let temp = tempfile::tempdir().unwrap();
    let mut repo = file(temp.path());
    let id = add_need(&mut repo);
    retitle(&id, "new").run(&mut repo).unwrap();
    undo("revert").run(&mut repo).unwrap();
    assert_eq!(repo.get(&id).unwrap().unwrap().title(), "a need");
    let reopened = FileStore::open_with(temp.path(), |_| Some("piko".into())).unwrap();
    let bytes = reopened.get(&id.to_string()).unwrap();
    let node: Node = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(node.title(), "a need");
}

#[test]
fn undo_says_whether_the_next_one_is_a_redo() {
    let mut repo = memory();
    add_need(&mut repo);
    let outcome = undo("mistake").run(&mut repo).unwrap();
    assert_eq!(outcome.next.len(), 1);
    assert!(outcome.next[0].contains("undo again"), "{:?}", outcome.next);
    let outcome = undo("again").run(&mut repo).unwrap();
    assert!(
        outcome.next[0].contains("this was a redo"),
        "{:?}",
        outcome.next
    );
}

#[test]
fn undo_keeps_the_why_and_source() {
    let mut repo = memory();
    add_need(&mut repo);
    undo("mistake").run(&mut repo).unwrap();
    let line = repo.store().history().last().unwrap();
    assert_eq!(line.why, "undo");
    assert_eq!(line.source, "mistake");
}
