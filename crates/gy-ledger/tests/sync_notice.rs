//! Refused writes are noticed until the writer touches the node again, and
//! undo stays with its writer (n-ecbf 2B2, ac-6f67/ac-08cc).
use gy_ledger::{
    FileStore, FormatVersion, Node, NodeId, NodeKind, Repository, Store, clear_rejected, format,
    handover, log, rejected_notices,
};
use std::path::Path;

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        id(NodeKind::Criterion, hash),
        "a",
        "2026-09-15T00:00:00Z",
        "an ac",
    )
    .unwrap()
}

fn event(seq: u64, actor: &str, by: Option<&str>, node: &str) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: actor.to_string(),
        why: "a write".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: by.map(str::to_string),
        by_mail: None,
        changes: vec![log::Change::Created {
            node: node.to_string(),
            value: serde_json::to_value(criterion("0001")).unwrap(),
        }],
    }
}

/// One rejected line, as the rebase would write it.
fn rejected_line(seq: u64, actor: &str, by: Option<&str>, node: &str, reason: &str) -> String {
    let rejected = serde_json::json!({
        "at": 0,
        "event": event(seq, actor, by, node),
        "reason": reason,
        "by": {"seq": 9, "actor": "peer"},
    });
    rejected.to_string()
}

fn ledger(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
}

fn add_rejected(dir: &Path, line: &str) {
    std::fs::write(dir.join("rejected.jsonl"), format!("{line}\n")).unwrap();
}

fn store(dir: &Path, actor: &str) -> FileStore {
    FileStore::open_with(dir, |_| Some(actor.into())).unwrap()
}

#[test]
fn a_refused_write_is_noticed() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    ledger(dir);
    add_rejected(
        dir,
        &rejected_line(2, "piko", None, "ac-0001", "the remote changed ac-0001"),
    );

    let notices = rejected_notices(dir);
    assert_eq!(notices.len(), 1);
    assert!(
        notices[0].contains("notice: your write seq 2"),
        "{}",
        notices[0]
    );
    assert!(notices[0].contains("the remote changed ac-0001"));
}

#[test]
fn handover_errors_carry_the_notice() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    ledger(dir);
    add_rejected(
        dir,
        &rejected_line(3, "piko", None, "ac-0001", "the remote deleted ac-0001"),
    );

    let report = handover(&Repository::new(store(dir, "piko")), None).unwrap();
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("notice: your write seq 3"))
    );
    let json = serde_json::to_value(&report).unwrap();
    assert!(json["errors"][0].as_str().unwrap().contains("notice"));
}

#[test]
fn a_write_on_a_touched_node_clears_the_notice() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    ledger(dir);
    add_rejected(
        dir,
        &rejected_line(
            2,
            "piko",
            Some("human"),
            "ac-0001",
            "the remote changed ac-0001",
        ),
    );

    // Another writer's move, or another node, keeps it.
    clear_rejected(dir, &event(9, "piko", Some("human"), "ac-0002")).unwrap();
    assert_eq!(rejected_notices(dir).len(), 1);
    clear_rejected(dir, &event(9, "other", Some("human"), "ac-0001")).unwrap();
    assert_eq!(rejected_notices(dir).len(), 1);

    // The same writer on the touched node clears it.
    clear_rejected(dir, &event(10, "piko", Some("human"), "ac-0001")).unwrap();
    assert!(rejected_notices(dir).is_empty());
}

#[test]
fn undo_only_your_own_last_write() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    ledger(dir);
    log::append(dir, &event(1, "piko", None, "ac-0001")).unwrap();

    let error = store(dir, "other").undo("undo", "test").unwrap_err();
    assert!(
        error.message.contains("undo only your own"),
        "{}",
        error.message
    );
    assert!(error.message.contains("piko"), "{}", error.message);

    // The same actor may undo it.
    assert!(store(dir, "piko").undo("undo", "test").is_ok());
}
