use gy_ledger::{Actor, Error, FormatVersion, MemoryStore, NodeId, NodeKind, Store};

fn store() -> MemoryStore {
    MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap())
}

#[test]
fn opening_without_an_actor_fails() {
    assert!(Actor::from_lookup(|_| None).is_err());
    assert!(Actor::new("   ").is_err());
    assert!(MemoryStore::open_with(FormatVersion::CURRENT, |_| None).is_err());
    assert!(MemoryStore::open_with(FormatVersion::CURRENT, |_| Some("piko".into())).is_ok());
}

#[test]
fn a_failed_transaction_writes_nothing() {
    let mut store = store();
    let result: Result<(), Error> = store.transaction(|staged| {
        staged.stage("a", "1");
        staged.stage("b", "2");
        Err(Error::invalid("stop"))
    });
    assert!(result.is_err());
    assert_eq!(store.get("a"), None);
    assert_eq!(store.get("b"), None);
}

#[test]
fn a_committed_transaction_is_visible() {
    let mut store = store();
    store
        .transaction(|staged| {
            staged.stage("a", "1");
            Ok(())
        })
        .unwrap();
    assert_eq!(store.get("a"), Some(&b"1"[..]));
}

#[test]
fn ids_carry_the_kind_prefix_and_a_short_hash() {
    let mut store = store();
    let id = NodeId::mint(NodeKind::Need, &mut store).unwrap();
    assert_eq!(id.kind(), NodeKind::Need);
    let text = id.to_string();
    assert!(
        text.starts_with("n-") && text.len() == "n-0000".len(),
        "{text}"
    );
}

#[test]
fn the_format_version_enters_at_birth_and_is_checked() {
    let store = store();
    assert_eq!(store.version(), FormatVersion::CURRENT);
    assert!(FormatVersion::CURRENT.supported());
    assert!(!FormatVersion(FormatVersion::CURRENT.0 + 1).supported());
}

#[test]
fn history_records_when_who_what_why_and_source() {
    let mut store = store();
    store
        .transaction(|staged| {
            staged.record("n-0001", "created", "the master asked", "conversation");
            Ok(())
        })
        .unwrap();
    let entry = &store.history()[0];
    assert_eq!(
        (entry.actor.name(), entry.node.as_str(), entry.what.as_str()),
        ("piko", "n-0001", "created")
    );
    assert_eq!(
        (entry.why.as_str(), entry.source.as_str()),
        ("the master asked", "conversation")
    );
    assert!(entry.at > 0);
}

#[test]
fn a_failed_transaction_keeps_no_history() {
    let mut store = store();
    let result: Result<(), Error> = store.transaction(|staged| {
        staged.record("n-0001", "created", "why", "source");
        Err(Error::invalid("stop"))
    });
    assert!(result.is_err());
    assert!(store.history().is_empty());
}

#[test]
fn a_committed_transaction_keeps_its_history() {
    let mut store = store();
    store
        .transaction(|staged| {
            staged.record("n-0001", "created", "why", "source");
            Ok(())
        })
        .unwrap();
    assert_eq!(store.history().len(), 1);
    assert_eq!(store.history()[0].node, "n-0001");
}
