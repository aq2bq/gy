use gy_ledger::{
    Actor, FormatVersion, MemoryStore, NeedAdd, Node, NodeData, NodeId, NodeKind, Operation,
    Repository,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

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

fn add(targets: Vec<NodeId>) -> NeedAdd {
    NeedAdd {
        scope: SCOPE.into(),
        title: "a need".into(),
        targets,
    }
}

#[test]
fn need_add_creates_a_need_with_both_sides() {
    let mut repo = repo();
    let ac = criterion("0001");
    repo.transaction("seed", "test", |repo| repo.put(&ac))
        .unwrap();
    let outcome = add(vec![ac.id().clone()]).run(&mut repo).unwrap();
    let node = repo.get(outcome.id.as_ref().unwrap()).unwrap().unwrap();
    assert_eq!(node.kind(), NodeKind::Need);
    assert_eq!(node.links().len(), 1);
    assert_eq!(outcome.changed, ["title", "scope", "created", "targets"]);
    match node.data() {
        NodeData::Need(data) => assert_eq!(data.targets, vec![ac.id().clone()]),
        _ => panic!("not a need"),
    }
}

#[test]
fn need_add_without_targets_is_rejected() {
    let mut repo = repo();
    assert!(add(vec![]).run(&mut repo).is_err());
    assert!(repo.all().unwrap().is_empty());
}

#[test]
fn need_add_with_a_missing_criterion_is_rejected() {
    let mut repo = repo();
    let missing = NodeId::from_hash(NodeKind::Criterion, "9999").unwrap();
    assert!(add(vec![missing]).run(&mut repo).is_err());
    assert!(repo.all().unwrap().is_empty());
}
