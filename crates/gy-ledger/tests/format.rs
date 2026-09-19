use gy_ledger::{
    Approval, FileStore, FormatVersion, Node, NodeData, NodeId, NodeKind, Repository,
    RequirementState, Revision, Store, format, location, log, next,
};

const DATE: &str = "2026-09-15T00:00:00Z";

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
        retries: 0,
        by: None,
        by_mail: None,
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
fn opening_a_version_two_ledger_raises_created_and_rebuilds_the_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion(2)).unwrap();

    // Two needs as version 2 wrote them: `created` is a bare date.
    let dated = |hash: &str, day: &str| {
        let node = node(hash, "a");
        let mut value = serde_json::to_value(&node).unwrap();
        value["created"] = serde_json::json!(day);
        value
    };
    let first = dated("0001", "2026-09-15");
    let second = dated("0002", "2026-09-15");
    // A snapshot in the old form: opening must drop it and rebuild from the log.
    let snapshot = serde_json::json!({
        "seq": 1,
        "nodes": {"n-0001": first.clone()},
    });
    std::fs::write(dir.join("snapshot.json"), snapshot.to_string()).unwrap();
    log::append(
        dir,
        &event(
            1,
            "seed",
            vec![
                log::Change::Created {
                    node: "n-0001".into(),
                    value: first,
                },
                log::Change::Created {
                    node: "n-0002".into(),
                    value: second,
                },
            ],
        ),
    )
    .unwrap();

    let repository = Repository::new(FileStore::open_with(dir, actor).unwrap());
    assert_eq!(repository.store().version(), FormatVersion::CURRENT);
    assert!(dir.join("format.2.bak").is_file());
    let id = NodeId::from_hash(NodeKind::Need, "0001").unwrap();
    let one = repository.get(&id).unwrap().unwrap();
    assert_eq!(one.created(), "2026-09-15T00:00:00Z");
    // The snapshot was rebuilt from the log, so every node carries the raised day.
    let rebuilt: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("snapshot.json")).unwrap()).unwrap();
    assert_eq!(
        rebuilt["nodes"]["n-0002"]["created"],
        "2026-09-15T00:00:00Z"
    );

    // next orders by created then id; the raised day keeps the id order.
    let ready: Vec<String> = next(&repository, None)
        .unwrap()
        .into_iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(ready, ["n-0001", "n-0002"]);
}

#[test]
fn every_old_instant_is_raised_on_read_with_and_without_a_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();

    let mut criterion = Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, "0001").unwrap(),
        "a",
        DATE,
        "a criterion",
    )
    .unwrap();
    if let NodeData::Criterion(data) = criterion.data_mut() {
        data.satisfied = true;
        data.satisfied_at = Some("2026-09-15".into());
    }
    let mut requirement = Node::requirement(
        NodeId::from_hash(NodeKind::Requirement, "0002").unwrap(),
        "a",
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    if let NodeData::Requirement(data) = requirement.data_mut() {
        data.approval = Some(Approval {
            design: "d".into(),
            heard_by: "h".into(),
            evidence: "e".into(),
            at: "2026-09-15".into(),
        });
        data.revisions.push(Revision {
            reason: "r".into(),
            source: "s".into(),
            at: "2026-09-15".into(),
        });
    }
    let criterion_value = serde_json::to_value(&criterion).unwrap();
    let requirement_value = serde_json::to_value(&requirement).unwrap();

    // A snapshot still holding the old, date-only instants.
    let snapshot = serde_json::json!({
        "seq": 1,
        "nodes": {"ac-0001": criterion_value.clone(), "r-0002": requirement_value.clone()},
    });
    std::fs::write(dir.join("snapshot.json"), snapshot.to_string()).unwrap();
    log::append(
        dir,
        &event(
            1,
            "seed",
            vec![
                log::Change::Created {
                    node: "ac-0001".into(),
                    value: criterion_value,
                },
                log::Change::Created {
                    node: "r-0002".into(),
                    value: requirement_value,
                },
            ],
        ),
    )
    .unwrap();
    // The snapshot path normalizes every instant it reads.
    let repository = Repository::new(FileStore::open_with(dir, actor).unwrap());
    raised_instants(&repository);

    // The event path normalizes the same way when the snapshot is gone.
    std::fs::remove_file(dir.join("snapshot.json")).unwrap();
    let repository = Repository::new(FileStore::open_with(dir, actor).unwrap());
    raised_instants(&repository);
}

fn raised_instants<S: Store>(repository: &Repository<S>) {
    let criterion = repository
        .get(&NodeId::from_hash(NodeKind::Criterion, "0001").unwrap())
        .unwrap()
        .unwrap();
    match criterion.data() {
        NodeData::Criterion(data) => {
            assert_eq!(data.satisfied_at.as_deref(), Some("2026-09-15T00:00:00Z"));
        }
        _ => panic!("not a criterion"),
    }
    let requirement = repository
        .get(&NodeId::from_hash(NodeKind::Requirement, "0002").unwrap())
        .unwrap()
        .unwrap();
    match requirement.data() {
        NodeData::Requirement(data) => {
            assert_eq!(data.approval.as_ref().unwrap().at, "2026-09-15T00:00:00Z");
            assert_eq!(data.revisions[0].at, "2026-09-15T00:00:00Z");
        }
        _ => panic!("not a requirement"),
    }
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
