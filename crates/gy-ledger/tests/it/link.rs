use gy_ledger::{
    Actor, DecisionScope, FormatVersion, MemoryStore, Node, NodeId, NodeKind, Operation, Relation,
    Repository, link::Link as LinkOp,
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

fn decision(hash: &str, body: &str) -> Node {
    let mut node = Node::decision(
        NodeId::from_hash(NodeKind::Decision, hash).unwrap(),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap();
    node.set_body(body);
    node
}

fn link(
    from: &NodeId,
    relation: Relation,
    to: &NodeId,
    mark: Option<&str>,
    remove: bool,
) -> LinkOp {
    LinkOp {
        from: from.clone(),
        relation,
        to: to.clone(),
        mark: mark.map(str::to_string),
        remove,
    }
}

fn seed(repo: &mut Repository<MemoryStore>, nodes: [&Node; 2]) {
    repo.transaction("seed", "test", |repo| {
        repo.put(nodes[0])?;
        repo.put(nodes[1])
    })
    .unwrap();
}

#[test]
fn link_adds_and_removes_an_edge() {
    let mut repo = repo();
    let need = need("0001");
    let criterion = criterion("0002");
    seed(&mut repo, [&need, &criterion]);
    let outcome = link(need.id(), Relation::Targets, criterion.id(), None, false)
        .run(&mut repo)
        .unwrap();
    assert_eq!(outcome.changed, ["linked"]);
    assert_eq!(repo.get(need.id()).unwrap().unwrap().links().len(), 1);
    assert_eq!(
        repo.incoming(criterion.id()).unwrap()[0].name(),
        "targeted-by"
    );

    let outcome = link(need.id(), Relation::Targets, criterion.id(), None, true)
        .run(&mut repo)
        .unwrap();
    assert_eq!(outcome.changed, ["unlinked"]);
    assert!(repo.get(need.id()).unwrap().unwrap().links().is_empty());
}

#[test]
fn a_duplicate_or_missing_edge_is_rejected() {
    let mut repo = repo();
    let need = need("0001");
    let criterion = criterion("0002");
    seed(&mut repo, [&need, &criterion]);
    assert!(
        link(need.id(), Relation::Targets, criterion.id(), None, true)
            .run(&mut repo)
            .is_err()
    );
    link(need.id(), Relation::Targets, criterion.id(), None, false)
        .run(&mut repo)
        .unwrap();
    assert!(
        link(need.id(), Relation::Targets, criterion.id(), None, false)
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn link_cannot_close_or_break_a_kind_pair() {
    let mut repo = repo();
    let need = need("0001");
    let decision = decision("0002", "body");
    seed(&mut repo, [&need, &decision]);
    assert!(
        link(need.id(), Relation::Closes, decision.id(), None, false)
            .run(&mut repo)
            .is_err()
    );
    assert!(
        link(
            need.id(),
            Relation::Narrows,
            decision.id(),
            Some("body"),
            false
        )
        .run(&mut repo)
        .is_err()
    );
}

#[test]
fn link_records_a_mark_on_narrows() {
    let mut repo = repo();
    let older = decision("0001", "Ordering may be implementation-dependent.");
    let newer = decision("0002", "body");
    seed(&mut repo, [&older, &newer]);
    let mark = "Ordering may be implementation-dependent.";
    link(newer.id(), Relation::Narrows, older.id(), Some(mark), false)
        .run(&mut repo)
        .unwrap();
    let links = repo.get(newer.id()).unwrap().unwrap().links().to_vec();
    assert_eq!(links[0].mark.as_deref(), Some(mark));
    assert_eq!(repo.incoming(older.id()).unwrap()[0].name(), "narrowed-by");
}
