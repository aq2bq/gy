//! r-489d: one reversible, closed graph component leaves every reading.
use gy_ledger::{
    Actor, Alias, Closed, ClosedBy, Closure, DecisionScope, FileStore, Filter, FormatVersion, Link,
    MemoryStore, Node, NodeData, NodeId, NodeKind, Operation, Relation, Repository,
    RequirementState, Retire, Store, Undo, file, format, handover, list, log, show,
};
use serde_json::Value;

const DATE: &str = "2026-11-05T00:00:00Z";
fn node(kind: NodeKind, hash: &str) -> Node {
    let id = NodeId::from_hash(kind, hash).unwrap();
    match kind {
        NodeKind::Decision => Node::decision(
            id,
            "a",
            DATE,
            "SECRET_TITLE",
            DecisionScope::recorded("SECRET_SCOPE").unwrap(),
        )
        .unwrap(),
        NodeKind::Need => {
            let mut n = Node::need(id, "a", DATE, "SECRET_NEED").unwrap();
            if let NodeData::Need(data) = n.data_mut() {
                data.closed = Some(Closed {
                    by: ClosedBy::Fact,
                    evidence: "SECRET_EVIDENCE".into(),
                });
            }
            n
        }
        NodeKind::Question => {
            let mut n = Node::question(id, "a", DATE, "SECRET_QUESTION").unwrap();
            if let NodeData::Question(data) = n.data_mut() {
                data.closure = Some(Closure::Fact);
                data.evidence = Some("SECRET_EVIDENCE".into());
            }
            n
        }
        NodeKind::Requirement => Node::requirement(
            id,
            "a",
            DATE,
            "SECRET_REQUIREMENT",
            RequirementState::Cancelled,
        )
        .unwrap(),
        NodeKind::Criterion => {
            let mut n = Node::criterion(id, "a", DATE, "SECRET_CRITERION").unwrap();
            if let NodeData::Criterion(data) = n.data_mut() {
                data.satisfied = true;
                data.evidence = Some("SECRET_EVIDENCE".into());
            }
            n
        }
    }
}
fn memory(nodes: &[Node]) -> Repository<MemoryStore> {
    let mut repo = Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("gy-dev-dev").unwrap(),
    ));
    // A historical fixture may contain orphaned satisfied criteria.
    let store = repo.store_mut();
    store.set_gate(std::sync::Arc::new(gy_ledger::Open));
    repo.transaction("SECRET_WHY", "SECRET_SOURCE", |r| {
        for n in nodes {
            r.put(n)?;
        }
        Ok(())
    })
    .unwrap();
    let store = repo.store_mut();
    store.set_gate(std::sync::Arc::new(gy_ledger::Rules));
    repo
}
fn file_repository(nodes: &[Node]) -> (tempfile::TempDir, Repository<FileStore>) {
    let tmp = tempfile::tempdir().unwrap();
    format::write(tmp.path(), FormatVersion::CURRENT).unwrap();
    let store = FileStore::open_with(tmp.path(), |_| Some("gy-dev-dev".into())).unwrap();
    let mut r = Repository::with_gate(store, std::sync::Arc::new(gy_ledger::Open));
    r.transaction("historical fixture", "test", |r| {
        for n in nodes {
            r.put(n)?;
        }
        Ok(())
    })
    .unwrap();
    let store = FileStore::open_with(tmp.path(), |_| Some("gy-dev-dev".into())).unwrap();
    (tmp, Repository::new(store))
}
fn retire<S: Store>(r: &mut Repository<S>, ids: &[&str], revoke: bool) -> gy_ledger::Result<()> {
    Retire {
        ids: ids.iter().map(|s| s.to_string()).collect(),
        reason: "earlier process records".into(),
        revoke,
    }
    .run(r)
    .map(|_| ())
}
fn readings<S: Store>(r: &Repository<S>) -> Value {
    let ids: Vec<String> = r
        .all()
        .unwrap()
        .iter()
        .map(|n| n.id().to_string())
        .collect();
    serde_json::json!({"list": list(r, &Filter::default()).unwrap(), "show": show(r, &ids, true).unwrap()})
}
#[test]
fn retirement_revoke_and_undo_preserve_readings_and_aliases() {
    let mut d = node(NodeKind::Decision, "0001");
    d.set_body("SECRET_BODY");
    d.add_alias(Alias("D-06".into()));
    let mut live = node(NodeKind::Decision, "0002");
    live.set_title("remaining").unwrap();
    let mut r = memory(&[d, live]);
    let before = readings(&r);
    let hand = handover(&r, None).unwrap();
    retire(&mut r, &["D-6"], false).unwrap();
    assert_eq!(r.all().unwrap().len(), 1);
    assert_eq!(
        handover(&r, None).unwrap().warnings.last().unwrap().count + 1,
        hand.warnings.last().unwrap().count
    );
    let hidden = show(&r, &["D-06".into()], true).unwrap_err().message;
    assert!(
        hidden.contains("seq 2") && hidden.contains("gy retire d-0001 --revoke"),
        "{hidden}"
    );
    assert!(!hidden.contains("SECRET"));
    retire(&mut r, &["D-06"], true).unwrap();
    assert_eq!(readings(&r), before);
    retire(&mut r, &["d-0001"], false).unwrap();
    Undo {
        reason: "restore".into(),
    }
    .run(&mut r)
    .unwrap();
    assert_eq!(readings(&r), before);
}
#[test]
fn all_thirteen_relations_refuse_both_boundaries_and_partial_restoration() {
    use NodeKind::{Criterion as C, Decision as D, Need as N, Question as Q, Requirement as R};
    let kinds = [
        (Q, D),
        (D, N),
        (D, D),
        (D, D),
        (D, D),
        (D, D),
        (N, C),
        (N, D),
        (N, R),
        (N, N),
        (R, D),
        (R, Q),
        (N, Q),
    ];
    for (relation, (a, b)) in Relation::ALL.into_iter().zip(kinds) {
        let mut from = node(a, "0001");
        let to = node(b, "0002");
        from.link(
            Link::new(from.id().clone(), relation, to.id().clone())
                .unwrap()
                .with_mark(Some("SECRET_SCOPE".into())),
        );
        let ids = [from.id().to_string(), to.id().to_string()];
        let (tmp, mut r) = file_repository(&[from, to]);
        let log_path = tmp.path().join(log::FILE);
        for id in &ids {
            let before = std::fs::read(&log_path).unwrap();
            let error = retire(&mut r, &[id], false).unwrap_err().message;
            for part in [&ids[0], relation.name(), &ids[1]] {
                assert!(error.contains(part), "{relation:?}: {error}");
            }
            assert_eq!(std::fs::read(&log_path).unwrap(), before);
        }
        retire(&mut r, &[&ids[0], &ids[1]], false).unwrap();
        assert!(r.all().unwrap().is_empty());
        let before = std::fs::read(&log_path).unwrap();
        let error = retire(&mut r, &[&ids[0]], true).unwrap_err().message;
        assert_eq!(std::fs::read(&log_path).unwrap(), before);
        assert!(error.contains("remaining edge"));
        retire(&mut r, &[&ids[0], &ids[1]], true).unwrap();
        assert_eq!(r.all().unwrap().len(), 2);
    }
}
#[test]
fn active_states_and_empty_selection_are_refused_without_history() {
    let mut open = Vec::new();
    open.push(
        Node::need(
            NodeId::from_hash(NodeKind::Need, "0001").unwrap(),
            "a",
            DATE,
            "need",
        )
        .unwrap(),
    );
    open.push(
        Node::question(
            NodeId::from_hash(NodeKind::Question, "0001").unwrap(),
            "a",
            DATE,
            "q",
        )
        .unwrap(),
    );
    open.push(
        Node::criterion(
            NodeId::from_hash(NodeKind::Criterion, "0001").unwrap(),
            "a",
            DATE,
            "ac",
        )
        .unwrap(),
    );
    for state in [RequirementState::Filed, RequirementState::Approved] {
        open.push(
            Node::requirement(
                NodeId::from_hash(
                    NodeKind::Requirement,
                    if state == RequirementState::Filed {
                        "0001"
                    } else {
                        "0002"
                    },
                )
                .unwrap(),
                "a",
                DATE,
                "r",
                state,
            )
            .unwrap(),
        );
    }
    let (tmp, mut r) = file_repository(&open);
    let bytes = std::fs::read(tmp.path().join(log::FILE)).unwrap();
    for n in &open {
        assert!(
            retire(&mut r, &[&n.id().to_string()], false)
                .unwrap_err()
                .message
                .contains("in progress")
        );
        assert_eq!(r.store().history().len(), open.len());
        assert_eq!(std::fs::read(tmp.path().join(log::FILE)).unwrap(), bytes);
    }
    assert!(retire(&mut r, &[], false).is_err());
}
#[test]
fn direct_writes_and_links_to_retired_nodes_are_refused_by_gate() {
    let hidden = node(NodeKind::Decision, "0001");
    let mut live = node(NodeKind::Decision, "0002");
    let mut r = memory(&[hidden.clone(), live.clone()]);
    retire(&mut r, &["d-0001"], false).unwrap();
    assert!(
        r.transaction("edit", "test", |r| r.put(&hidden))
            .unwrap_err()
            .message
            .contains("retired")
    );
    live.link(Link::new(live.id().clone(), Relation::Completes, hidden.id().clone()).unwrap());
    assert!(
        r.transaction("link", "test", |r| r.put(&live))
            .unwrap_err()
            .message
            .contains("retired")
    );
    assert_eq!(r.all().unwrap().len(), 1);
}
#[test]
fn file_retirement_is_one_append_and_survives_replay_snapshot_and_past_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let actor = |_: &str| Some("gy-dev-dev".into());
    let mut r = Repository::new(FileStore::open_with(dir, actor).unwrap());
    let nodes = [
        node(NodeKind::Decision, "0001"),
        node(NodeKind::Decision, "0002"),
    ];
    r.transaction("seed", "SECRET_SOURCE", |r| {
        for n in &nodes {
            r.put(n)?;
        }
        Ok(())
    })
    .unwrap();
    let before = readings(&r);
    let bytes = std::fs::read(dir.join(log::FILE)).unwrap();
    retire(&mut r, &["d-0001", "d-0002"], false).unwrap();
    let after = std::fs::read(dir.join(log::FILE)).unwrap();
    assert!(after.starts_with(&bytes));
    assert_eq!(
        after[bytes.len()..].iter().filter(|b| **b == b'\n').count(),
        1
    );
    assert_eq!(format::read(dir).unwrap(), FormatVersion::RETIREMENT);
    assert!(r.all().unwrap().is_empty());
    for seq in [0, 1, 2, 99] {
        let at = Repository::new(file::open_at(dir, seq, actor).unwrap());
        assert!(at.all().unwrap().is_empty(), "seq {seq}");
    }
    drop(r);
    for snapshot in [true, false] {
        if !snapshot {
            std::fs::remove_file(dir.join("snapshot.json")).unwrap();
        }
        let r = Repository::new(FileStore::open_with(dir, actor).unwrap());
        assert!(r.all().unwrap().is_empty());
    }
    let mut r = Repository::new(FileStore::open_with(dir, actor).unwrap());
    Undo {
        reason: "restore".into(),
    }
    .run(&mut r)
    .unwrap();
    assert_eq!(readings(&r), before);
    let past = Repository::new(file::open_at(dir, 2, actor).unwrap());
    assert_eq!(readings(&past), before);
}

#[test]
fn surviving_prose_references_are_preserved_but_structural_ids_are_excluded() {
    let hidden = node(NodeKind::Decision, "0001");
    let mut live = node(NodeKind::Decision, "0002");
    live.set_title("remaining").unwrap();
    live.set_body("This text mentions d-0001; it must stay verbatim.");
    let mut r = memory(&[hidden, live.clone()]);
    r.transaction("reason mentions d-0001", "a reference to d-0001", |r| {
        r.put(&live)
    })
    .unwrap();
    let before = show(&r, &["d-0002".into()], true).unwrap();
    retire(&mut r, &["d-0001"], false).unwrap();
    let after = show(&r, &["d-0002".into()], true).unwrap();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(after).unwrap()
    );
    assert_eq!(
        r.all()
            .unwrap()
            .iter()
            .map(|n| n.id().to_string())
            .collect::<Vec<_>>(),
        vec!["d-0002"]
    );
    assert!(r.history().unwrap().iter().all(|row| row.node != "d-0001"));
    assert!(
        r.history()
            .unwrap()
            .iter()
            .any(|row| row.why == "reason mentions d-0001")
    );
    assert!(
        r.store().keys().contains(&"d-0001".into()),
        "retired IDs stay reserved"
    );
}
