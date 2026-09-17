use gy_ledger::{
    Actor, DecisionScope, FormatVersion, MemoryStore, NeedAdd, Node, NodeId, NodeKind, Operation,
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

fn decision(hash: &str) -> Node {
    Node::decision(
        NodeId::from_hash(NodeKind::Decision, hash).unwrap(),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap()
}

fn add(targets: Vec<NodeId>, spawned_by: Option<NodeId>) -> NeedAdd {
    NeedAdd {
        scope: SCOPE.into(),
        title: "a need".into(),
        targets,
        spawned_by,
        body: None,
    }
}

#[test]
fn need_add_creates_a_need_and_its_reverse_edges() {
    let mut repo = repo();
    let ac = criterion("0001");
    repo.transaction("seed", "test", |repo| repo.put(&ac))
        .unwrap();
    let outcome = add(vec![ac.id().clone()], None).run(&mut repo).unwrap();
    let id = outcome.id.clone().unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.links().len(), 1);
    assert_eq!(node.links()[0].name(), "targets");
    let incoming = repo.incoming(ac.id()).unwrap();
    assert_eq!(incoming.len(), 1);
    assert_eq!((incoming[0].name(), &incoming[0].to), ("targeted-by", &id));
    assert_eq!(outcome.changed, ["created", "targets"]);
}

#[test]
fn need_add_links_a_spawning_decision() {
    let mut repo = repo();
    let ac = criterion("0001");
    let decision = decision("0002");
    repo.transaction("seed", "test", |repo| {
        repo.put(&ac)?;
        repo.put(&decision)
    })
    .unwrap();
    let outcome = add(vec![ac.id().clone()], Some(decision.id().clone()))
        .run(&mut repo)
        .unwrap();
    let node = repo.get(outcome.id.as_ref().unwrap()).unwrap().unwrap();
    assert_eq!(node.links().len(), 2);
    assert_eq!(outcome.changed, ["created", "targets", "spawned-by"]);
}

#[test]
fn need_add_without_targets_is_rejected() {
    let mut repo = repo();
    assert!(add(vec![], None).run(&mut repo).is_err());
    assert!(repo.all().unwrap().is_empty());
}

#[test]
fn need_add_with_a_missing_criterion_is_rejected() {
    let mut repo = repo();
    let missing = NodeId::from_hash(NodeKind::Criterion, "9999").unwrap();
    assert!(add(vec![missing], None).run(&mut repo).is_err());
    assert!(repo.all().unwrap().is_empty());
}

#[test]
fn need_add_with_a_non_decision_spawn_is_rejected() {
    let mut repo = repo();
    let ac = criterion("0001");
    repo.transaction("seed", "test", |repo| repo.put(&ac))
        .unwrap();
    assert!(
        add(vec![ac.id().clone()], Some(ac.id().clone()))
            .run(&mut repo)
            .is_err()
    );
}
