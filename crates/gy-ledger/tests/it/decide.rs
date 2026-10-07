use gy_ledger::{
    Actor, Decide, DecisionScope, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind,
    Operation, Relation, Repository,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn question(hash: &str) -> Node {
    Node::question(
        NodeId::from_hash(NodeKind::Question, hash).unwrap(),
        SCOPE,
        DATE,
        "a question",
    )
    .unwrap()
}

fn older(hash: &str, body: &str) -> Node {
    let mut node = Node::decision(
        NodeId::from_hash(NodeKind::Decision, hash).unwrap(),
        SCOPE,
        DATE,
        "older decision",
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap();
    node.set_body(body);
    node
}

fn decide(closes: Vec<NodeId>, relates: Vec<(Relation, NodeId, Option<String>)>) -> Decide {
    Decide {
        scope: SCOPE.into(),
        title: "a decision".into(),
        decision_scope: DecisionScope::recorded("applies in production").unwrap(),
        body: None,
        source: None,
        closes,
        relates,
    }
}

#[test]
fn decide_creates_a_decision_without_an_adr() {
    let mut repo = repo();
    let outcome = decide(vec![], vec![]).run(&mut repo).unwrap();
    let node = repo.get(outcome.id.as_ref().unwrap()).unwrap().unwrap();
    assert_eq!(node.kind(), NodeKind::Decision);
    assert_eq!(outcome.changed, ["created"]);

    let mut unrecorded = decide(vec![], vec![]);
    unrecorded.decision_scope = DecisionScope::migration_unrecorded();
    assert!(unrecorded.run(&mut repo).is_err());
}

#[test]
fn decide_closes_a_question_and_its_edge() {
    let mut repo = repo();
    let question = question("0001");
    repo.transaction("seed", "test", |repo| repo.put(&question))
        .unwrap();
    let outcome = decide(vec![question.id().clone()], vec![])
        .run(&mut repo)
        .unwrap();
    let decision = outcome.id.clone().unwrap();
    let closed = repo.get(question.id()).unwrap().unwrap();
    match closed.data() {
        NodeData::Question(data) => {
            assert!(data.closure.is_some());
            assert_eq!(
                data.evidence.as_deref(),
                Some(decision.to_string().as_str())
            );
        }
        _ => panic!("not a question"),
    }
    assert_eq!(repo.incoming(&decision).unwrap()[0].name(), "closed-by");
    assert!(outcome.changed.contains(&"closes".to_string()));
}

#[test]
fn closing_a_non_question_is_rejected() {
    let mut repo = repo();
    let older = older("0001", "body");
    repo.transaction("seed", "test", |repo| repo.put(&older))
        .unwrap();
    assert!(
        decide(vec![older.id().clone()], vec![])
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn narrows_needs_a_mark() {
    let mut repo = repo();
    let older = older("0001", "Ordering may be implementation-dependent.");
    repo.transaction("seed", "test", |repo| repo.put(&older))
        .unwrap();
    assert!(
        decide(vec![], vec![(Relation::Narrows, older.id().clone(), None)])
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn a_mark_that_is_not_in_the_body_is_rejected() {
    let mut repo = repo();
    let older = older("0001", "Ordering may be implementation-dependent.");
    repo.transaction("seed", "test", |repo| repo.put(&older))
        .unwrap();
    let error = decide(
        vec![],
        vec![(
            Relation::Narrows,
            older.id().clone(),
            Some("not in the body".into()),
        )],
    )
    .run(&mut repo)
    .unwrap_err()
    .to_string();
    assert!(error.contains("not in the body"), "{error}");
    assert!(repo.get(older.id()).unwrap().unwrap().links().is_empty());
}

#[test]
fn supersedes_is_visible_from_the_older_decision() {
    let mut repo = repo();
    let older = older("0001", "Old behavior");
    repo.transaction("seed", "test", |repo| repo.put(&older))
        .unwrap();
    let outcome = decide(
        vec![],
        vec![(
            Relation::Supersedes,
            older.id().clone(),
            Some("Old behavior".into()),
        )],
    )
    .run(&mut repo)
    .unwrap();
    let incoming = repo.incoming(older.id()).unwrap();
    assert_eq!(incoming[0].name(), "superseded-by");
    assert_eq!(incoming[0].to, outcome.id.clone().unwrap());
    assert_eq!(incoming[0].mark.as_deref(), Some("Old behavior"));
}

#[test]
fn a_failed_decide_writes_nothing() {
    let mut repo = repo();
    let question = question("0001");
    let older = older("0002", "Old behavior");
    repo.transaction("seed", "test", |repo| {
        repo.put(&question)?;
        repo.put(&older)
    })
    .unwrap();
    let result = decide(
        vec![question.id().clone()],
        vec![(Relation::Narrows, older.id().clone(), None)],
    )
    .run(&mut repo);
    assert!(result.is_err());
    match repo.get(question.id()).unwrap().unwrap().data() {
        NodeData::Question(data) => assert!(data.closure.is_none()),
        _ => panic!("not a question"),
    }
    assert_eq!(repo.all().unwrap().len(), 2);
}
