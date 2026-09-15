use gy_ledger::{
    Actor, FileStore, FormatVersion, MemoryStore, Node, NodeId, NodeKind, Operation, Repository,
    ScopeRename, Store, format, log,
};

const DATE: &str = "2026-09-15";

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

fn repo(scopes: &[&str]) -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
    .with_scopes(scopes.iter().map(|scope| (*scope).to_string()).collect())
}

fn need(hash: &str, scope: &str) -> Node {
    Node::need(
        NodeId::from_hash(NodeKind::Need, hash).unwrap(),
        scope,
        DATE,
        "a need",
    )
    .unwrap()
}

fn seed<S: Store>(repo: &mut Repository<S>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn rename(from: &str, to: &str) -> ScopeRename {
    ScopeRename {
        from: from.to_string(),
        to: to.to_string(),
    }
}

#[test]
fn scope_rename_moves_every_node_and_leaves_one_history_line() {
    let mut repo = repo(&["old"]);
    let keep = need("0001", "old");
    let keep_id = keep.id().clone();
    let other = need("0002", "elsewhere");
    let other_id = other.id().clone();
    seed(&mut repo, &[keep, other]);

    let outcome = rename("old", "new").run(&mut repo).unwrap();
    assert_eq!(outcome.changed, ["scope: old → new (1 nodes)"]);
    assert_eq!(outcome.value, 1);
    assert_eq!(repo.get(&keep_id).unwrap().unwrap().scope(), "new");
    assert_eq!(repo.get(&other_id).unwrap().unwrap().scope(), "elsewhere");

    let last = repo.store().history().last().unwrap();
    assert_eq!(last.what, "scope renamed old → new (1 nodes)");
    assert_eq!(last.node, "");
}

#[test]
fn scope_rename_rejects_bad_names() {
    let mut repo = repo(&["old", "taken"]);
    let node = need("0001", "old");
    let id = node.id().clone();
    seed(&mut repo, &[node]);

    assert!(rename("missing", "new").run(&mut repo).is_err());
    assert!(rename("old", "taken").run(&mut repo).is_err());
    assert!(rename("old", "a/b").run(&mut repo).is_err());
    assert!(rename("old", "").run(&mut repo).is_err());
    assert_eq!(repo.get(&id).unwrap().unwrap().scope(), "old");
}

#[test]
fn undo_reverses_a_scope_rename() {
    let mut repo = repo(&["old"]);
    let node = need("0001", "old");
    let id = node.id().clone();
    seed(&mut repo, &[node]);

    rename("old", "new").run(&mut repo).unwrap();
    assert_eq!(repo.get(&id).unwrap().unwrap().scope(), "new");

    repo.store_mut().undo("undo", "test").unwrap();
    assert_eq!(repo.get(&id).unwrap().unwrap().scope(), "old");
}

#[test]
fn a_scope_rename_is_one_log_change_and_survives_reopen() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let mut repo = Repository::new(FileStore::open_with(dir, actor).unwrap())
        .with_scopes(vec!["old".to_string()]);
    let node = need("0001", "old");
    let id = node.id().clone();
    seed(&mut repo, &[node]);

    let outcome = rename("old", "new").run(&mut repo).unwrap();
    assert_eq!(outcome.value, 1);
    drop(repo);

    let (events, _) = log::read(dir).unwrap();
    let last = events.last().unwrap();
    assert_eq!(last.changes.len(), 1);
    assert!(matches!(last.changes[0], log::Change::ScopeRenamed { .. }));

    let reopened = Repository::new(FileStore::open_with(dir, actor).unwrap());
    assert_eq!(reopened.get(&id).unwrap().unwrap().scope(), "new");

    // A full replay without the snapshot reaches the same result.
    std::fs::remove_file(dir.join("snapshot.json")).unwrap();
    let replayed = Repository::new(FileStore::open_with(dir, actor).unwrap());
    assert_eq!(replayed.get(&id).unwrap().unwrap().scope(), "new");
}
