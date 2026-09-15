use gy_ledger::{
    FileStore, FormatVersion, Node, NodeId, NodeKind, Repository, Store, format, location, log,
};

const DATE: &str = "2026-09-15";

fn actor(_: &str) -> Option<String> {
    Some("piko".to_string())
}

fn event(seq: u64, why: &str, changes: Vec<log::Change>) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: "piko".to_string(),
        why: why.to_string(),
        source: "test".to_string(),
        changes,
    }
}

fn node(hash: &str, scope: &str) -> Node {
    Node::need(
        NodeId::from_hash(NodeKind::Need, hash).unwrap(),
        scope,
        DATE,
        "a need",
    )
    .unwrap()
}

fn created(node: &Node) -> log::Change {
    log::Change::Created {
        node: node.id().to_string(),
        value: serde_json::to_value(node).unwrap(),
    }
}

#[test]
fn a_missing_format_file_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    assert!(format::read(temp.path()).is_err());
}

#[test]
fn a_newer_format_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    format::write(temp.path(), FormatVersion(FormatVersion::CURRENT.0 + 1)).unwrap();
    assert!(format::read(temp.path()).is_err());
    assert!(!FormatVersion(FormatVersion::CURRENT.0 + 1).supported());
}

#[test]
fn the_current_format_opens() {
    let temp = tempfile::tempdir().unwrap();
    format::write(temp.path(), FormatVersion::CURRENT).unwrap();
    assert_eq!(format::read(temp.path()).unwrap(), FormatVersion::CURRENT);
}

#[test]
fn an_unknown_migration_jump_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion(0)).unwrap();
    assert!(format::migrate(dir, FormatVersion(0), FormatVersion::CURRENT).is_err());
    assert!(format::migrate(dir, FormatVersion::CURRENT, FormatVersion::CURRENT).is_ok());
}

#[test]
fn opening_a_version_one_ledger_keeps_a_copy_and_raises_the_format() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion(1)).unwrap();
    log::append(dir, &event(1, "seed", vec![created(&node("0001", "a"))])).unwrap();

    let store = FileStore::open_with(dir, actor).unwrap();
    assert_eq!(store.version(), FormatVersion::CURRENT);
    assert!(dir.join("format.1.bak").is_file());
    assert!(dir.join("events.jsonl.1.bak").is_file());
    assert_eq!(format::read(dir).unwrap(), FormatVersion::CURRENT);
}

#[test]
fn a_scope_rename_replays_from_the_whole_log() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    log::append(dir, &event(1, "seed", vec![created(&node("0001", "old"))])).unwrap();
    log::append(
        dir,
        &event(
            2,
            "scope rename old new",
            vec![log::Change::ScopeRenamed {
                from: "old".to_string(),
                to: "new".to_string(),
                nodes: 1,
            }],
        ),
    )
    .unwrap();
    let id = NodeId::from_hash(NodeKind::Need, "0001").unwrap();

    let repository = Repository::new(FileStore::open_with(dir, actor).unwrap());
    assert_eq!(repository.get(&id).unwrap().unwrap().scope(), "new");

    // A full replay without the snapshot reaches the same result.
    std::fs::remove_file(dir.join("snapshot.json")).unwrap();
    let repository = Repository::new(FileStore::open_with(dir, actor).unwrap());
    assert_eq!(repository.get(&id).unwrap().unwrap().scope(), "new");
}

#[test]
fn a_corrupt_format_file_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join(format::FILE), "not a version\n").unwrap();
    assert!(format::read(temp.path()).is_err());
}

#[test]
fn the_location_key_is_stable_and_root_specific() {
    let temp = tempfile::tempdir().unwrap();
    let one = temp.path().join("one");
    let two = temp.path().join("two");
    std::fs::create_dir_all(&one).unwrap();
    std::fs::create_dir_all(&two).unwrap();
    assert_eq!(location::key(&one), location::key(&one));
    assert_ne!(location::key(&one), location::key(&two));
    assert_eq!(
        location::dir_in(temp.path(), &one),
        temp.path().join("gy").join(location::key(&one))
    );
    assert_eq!(location::key(&one).len(), 8);
}
