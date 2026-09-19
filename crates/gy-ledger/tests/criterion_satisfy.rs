use gy_ledger::{
    Actor, CriterionSatisfy, FormatVersion, Link, MemoryStore, Node, NodeData, NodeId, NodeKind,
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

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        SCOPE,
        DATE,
        "a criterion",
    )
    .unwrap()
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

fn seed(repo: &mut Repository<MemoryStore>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn satisfy(id: &NodeId, evidence: &str, revoke: bool) -> CriterionSatisfy {
    CriterionSatisfy {
        id: id.clone(),
        evidence: evidence.into(),
        revoke,
    }
}

#[test]
fn criterion_satisfy_records_evidence_and_the_date() {
    let mut repo = repo();
    let criterion = criterion("0001");
    repo.transaction("seed", "test", |repo| repo.put(&criterion))
        .unwrap();
    let outcome = satisfy(criterion.id(), "verified in production", false)
        .run(&mut repo)
        .unwrap();
    let node = repo.get(criterion.id()).unwrap().unwrap();
    match node.data() {
        NodeData::Criterion(data) => {
            assert!(data.satisfied);
            assert_eq!(data.evidence.as_deref(), Some("verified in production"));
            let at = data.satisfied_at.as_deref().unwrap();
            assert!(at.len() == 20 && at.ends_with('Z'), "{at}");
        }
        _ => panic!("not a criterion"),
    }
    assert_eq!(outcome.changed, ["satisfied"]);
    let entry = repo.store().history().last().unwrap();
    assert_eq!(entry.why, format!("criterion satisfy {}", criterion.id()));
    assert_eq!(entry.source, "verified in production");
}

#[test]
fn satisfying_twice_is_rejected() {
    let mut repo = repo();
    let criterion = criterion("0001");
    repo.transaction("seed", "test", |repo| repo.put(&criterion))
        .unwrap();
    satisfy(criterion.id(), "first", false)
        .run(&mut repo)
        .unwrap();
    assert!(
        satisfy(criterion.id(), "again", false)
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn revoke_clears_satisfied_and_keeps_evidence() {
    let mut repo = repo();
    let criterion = criterion("0001");
    repo.transaction("seed", "test", |repo| repo.put(&criterion))
        .unwrap();
    satisfy(criterion.id(), "was true", false)
        .run(&mut repo)
        .unwrap();
    satisfy(criterion.id(), "no longer true", true)
        .run(&mut repo)
        .unwrap();
    let node = repo.get(criterion.id()).unwrap().unwrap();
    match node.data() {
        NodeData::Criterion(data) => {
            assert!(!data.satisfied);
            assert_eq!(data.evidence.as_deref(), Some("no longer true"));
            assert!(data.satisfied_at.is_none());
        }
        _ => panic!("not a criterion"),
    }
}

#[test]
fn satisfying_a_non_criterion_is_rejected() {
    let mut repo = repo();
    let need = Node::need(
        NodeId::from_hash(NodeKind::Need, "0002").unwrap(),
        SCOPE,
        DATE,
        "a need",
    )
    .unwrap();
    repo.transaction("seed", "test", |repo| repo.put(&need))
        .unwrap();
    assert!(
        satisfy(need.id(), "evidence", false)
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn criterion_satisfy_points_at_the_other_criterion() {
    let mut repo = repo();
    let mut need = need("0001");
    let need_id = need.id().clone();
    let first = criterion("0002");
    let first_id = first.id().clone();
    let second = criterion("0003");
    let second_id = second.id().clone();
    need.link(Link::new(need_id.clone(), Relation::Targets, first_id.clone()).unwrap());
    need.link(Link::new(need_id, Relation::Targets, second_id.clone()).unwrap());
    seed(&mut repo, &[need, first, second]);

    let satisfied = satisfy(&first_id, "verified", false)
        .run(&mut repo)
        .unwrap();
    assert!(satisfied.missing.is_empty());
    assert_eq!(satisfied.next, [format!("criterion satisfy {second_id}")]);

    let last = satisfy(&second_id, "verified", false)
        .run(&mut repo)
        .unwrap();
    assert!(last.next.is_empty());
}
