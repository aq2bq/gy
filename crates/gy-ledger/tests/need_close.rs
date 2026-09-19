use gy_ledger::{
    Actor, ClosedBy, FormatVersion, Link, MemoryStore, NeedClose, Node, NodeData, NodeId, NodeKind,
    Operation, Relation, Repository, Store,
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

fn seed<S: Store>(repo: &mut Repository<S>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn target(need: &mut Node, criterion: &Node) {
    need.link(Link::new(need.id().clone(), Relation::Targets, criterion.id().clone()).unwrap());
}

fn satisfied(hash: &str) -> Node {
    let mut node = criterion(hash);
    if let NodeData::Criterion(data) = node.data_mut() {
        data.satisfied = true;
    }
    node
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

#[test]
fn closed_need_reports_an_unmet_criterion_no_open_need_bears() {
    let mut repo = repo();
    let ac = criterion("0001");
    let ac_id = ac.id().clone();
    let mut node = need("0002");
    target(&mut node, &ac);
    let id = node.id().clone();
    seed(&mut repo, &[ac, node]);

    let outcome = close(&id, ClosedBy::Fact, "resolved")
        .run(&mut repo)
        .unwrap();
    assert_eq!(outcome.missing, [format!("unmet criterion {ac_id}")]);
    assert_eq!(
        outcome.next,
        [format!("criterion satisfy {ac_id} --evidence …")]
    );
}

#[test]
fn closed_need_omits_a_criterion_an_open_need_still_bears() {
    let mut repo = repo();
    let ac = criterion("0001");
    let mut first = need("0002");
    target(&mut first, &ac);
    let first_id = first.id().clone();
    let mut second = need("0003");
    target(&mut second, &ac);
    seed(&mut repo, &[ac, first, second]);

    let outcome = close(&first_id, ClosedBy::Fact, "resolved")
        .run(&mut repo)
        .unwrap();
    assert!(outcome.missing.is_empty(), "{:?}", outcome.missing);
    assert!(outcome.next.is_empty());
}

#[test]
fn closed_need_omits_a_satisfied_criterion() {
    let mut repo = repo();
    let ac = satisfied("0001");
    let mut node = need("0002");
    target(&mut node, &ac);
    let id = node.id().clone();
    seed(&mut repo, &[ac, node]);

    let outcome = close(&id, ClosedBy::Fact, "resolved")
        .run(&mut repo)
        .unwrap();
    assert!(outcome.missing.is_empty(), "{:?}", outcome.missing);
    assert!(outcome.next.is_empty());
}
