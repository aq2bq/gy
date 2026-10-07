use gy_ledger::link::Link;
use gy_ledger::{
    CriterionAdd, Edit, FileStore, FormatVersion, MemoryStore, NeedAdd, NodeId, Operation,
    Relation, Repository, Store, Undo, file, format,
};
use std::path::Path;

/// Six writes on a real ledger directory: a criterion, two needs, a link, an
/// edit, and an undo of that edit.
fn ledger(dir: &Path) -> (Repository<FileStore>, NodeId) {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let store = FileStore::open_with(dir, |_| Some("piko".to_string())).unwrap();
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    let criterion = CriterionAdd {
        scope: "a".to_string(),
        title: "measurable".to_string(),
        body: None,
    }
    .run(&mut repo)
    .unwrap()
    .id
    .unwrap();
    let first = NeedAdd {
        scope: "a".to_string(),
        title: "first".to_string(),
        targets: vec![criterion.clone()],
        spawned_by: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap()
    .id
    .unwrap();
    let second = NeedAdd {
        scope: "a".to_string(),
        title: "second".to_string(),
        targets: vec![criterion],
        spawned_by: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap()
    .id
    .unwrap();
    Link {
        from: second,
        relation: Relation::DependsOn,
        to: first.clone(),
        mark: None,
        remove: false,
    }
    .run(&mut repo)
    .unwrap();
    Edit {
        id: first.clone(),
        reason: "tidy".to_string(),
        title: Some("first, tidied".to_string()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();
    Undo {
        reason: "wrong".to_string(),
    }
    .run(&mut repo)
    .unwrap();
    (repo, first)
}

fn read(dir: &Path, seq: u64) -> Repository<MemoryStore> {
    Repository::new(file::open_at(dir, seq, |_| Some("piko".to_string())).unwrap())
}

#[test]
fn open_at_replays_up_to_a_sequence() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    let (now, first) = ledger(dir);
    let last = now.store().history().last().unwrap().seq;

    // The nodes and the history only grow as the sequence advances.
    let (mut nodes, mut entries) = (0, 0);
    for seq in 0..=last {
        let store = read(dir, seq);
        assert!(store.store().keys().len() >= nodes, "nodes fell at {seq}");
        assert!(
            store.store().history().len() >= entries,
            "history fell at {seq}"
        );
        nodes = store.store().keys().len();
        entries = store.store().history().len();
    }

    // The end is the ledger itself, and the writes came to three nodes. A
    // fresh open replays the log, so it is the shape to compare with (the
    // still-open store records `put` for what this session wrote).
    let reopened = FileStore::open_with(dir, |_| Some("piko".to_string())).unwrap();
    let end = read(dir, last);
    assert_eq!(end.store().keys(), reopened.keys());
    assert_eq!(end.store().history(), reopened.history());
    assert_eq!(end.store().keys().len(), 3);

    // The undo gives the old title back at its own sequence, not before it.
    let undo = now
        .store()
        .history()
        .iter()
        .find(|entry| entry.why == "undo")
        .unwrap()
        .seq;
    let title = |seq: u64| {
        read(dir, seq)
            .get(&first)
            .unwrap()
            .unwrap()
            .title()
            .to_string()
    };
    assert_eq!(title(undo - 1), "first, tidied");
    assert_eq!(title(undo), "first");
}
