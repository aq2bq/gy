//! Batch reads preserve the single-question judgement, including cross-scope
//! waits, raised edges, and closed questions.
use gy_ledger::{
    Actor, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind, Operation, QuestionClose,
    Relation, Repository, unwaited, unwaited_questions,
};

#[test]
fn batch_matches_individual_questions_after_a_close() {
    let id = |kind, n: usize| NodeId::from_hash(kind, format!("{n:04x}")).unwrap();
    let mut nodes: Vec<Node> = (0..50)
        .map(|n| {
            Node::question(
                id(NodeKind::Question, n),
                "a",
                "2026-10-30T00:00:00Z",
                "a question",
            )
            .unwrap()
        })
        .collect();
    let mut need = Node::need(
        id(NodeKind::Need, 0),
        "b",
        "2026-10-30T00:00:00Z",
        "cross-scope waiter",
    )
    .unwrap();
    need.link(Link::new(need.id().clone(), Relation::WaitsOn, nodes[0].id().clone()).unwrap());
    nodes.push(need);
    let mut req = Node::requirement(
        id(NodeKind::Requirement, 0),
        "b",
        "2026-10-30T00:00:00Z",
        "raised it",
        gy_ledger::RequirementState::Filed,
    )
    .unwrap();
    req.link(Link::new(req.id().clone(), Relation::Raised, nodes[1].id().clone()).unwrap());
    nodes.push(req);
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("test").unwrap());
    let mut repo = Repository::new(store);
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    QuestionClose {
        id: id(NodeKind::Question, 2),
        by: gy_ledger::Closure::Fact,
        evidence: "settled".into(),
        decision: None,
    }
    .run(&mut repo)
    .unwrap();
    let all = repo.all().unwrap();
    let batch = unwaited_questions(&all);
    assert_eq!(batch.len(), 47);
    for node in &all {
        assert_eq!(batch.contains(&node.id().to_string()), unwaited(node, &all));
    }
}
