use gy_ledger::{
    Actor, ClosedBy, FormatVersion, MemoryStore, NeedClose, Node, NodeData, NodeId, NodeKind,
    Operation, Repository,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn need(hash: &str) -> Node {
    Node::need(
        NodeId::from_hash(NodeKind::Need, hash).unwrap(),
        SCOPE,
        DATE,
        "a need",
    )
    .unwrap()
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        SCOPE,
        DATE,
        "a criterion",
    )
    .unwrap()
}

fn close(id: &NodeId, by: ClosedBy, evidence: &str) -> NeedClose {
    NeedClose {
        id: id.clone(),
        by,
        evidence: evidence.into(),
    }
}

#[test]
fn need_close_records_the_closure() {
    let mut repo = repo();
    let node = need("0001");
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    let outcome = close(node.id(), ClosedBy::External, "closed externally")
        .run(&mut repo)
        .unwrap();
    let closed = repo.get(node.id()).unwrap().unwrap();
    match closed.data() {
        NodeData::Need(data) => assert_eq!(data.closed.as_ref().unwrap().by, ClosedBy::External),
        _ => panic!("not a need"),
    }
    assert_eq!(outcome.changed, ["closed"]);
}

#[test]
fn closing_an_already_closed_need_is_rejected() {
    let mut repo = repo();
    let node = need("0001");
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    close(node.id(), ClosedBy::Fact, "done")
        .run(&mut repo)
        .unwrap();
    assert!(
        close(node.id(), ClosedBy::Fact, "done")
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn closing_without_evidence_is_rejected() {
    let mut repo = repo();
    let node = need("0001");
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    assert!(
        close(node.id(), ClosedBy::Fact, "  ")
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn closing_a_non_need_is_rejected() {
    let mut repo = repo();
    let ac = criterion("0001");
    repo.transaction("seed", "test", |repo| repo.put(&ac))
        .unwrap();
    assert!(
        close(ac.id(), ClosedBy::Fact, "done")
            .run(&mut repo)
            .is_err()
    );
}
