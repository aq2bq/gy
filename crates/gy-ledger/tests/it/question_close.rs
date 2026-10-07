use gy_ledger::{
    Actor, Closure, DecisionScope, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind,
    Operation, QuestionClose, Repository, Store,
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

fn close(id: &NodeId, by: Closure, decision: Option<NodeId>) -> QuestionClose {
    QuestionClose {
        id: id.clone(),
        by,
        evidence: "closing evidence".into(),
        decision,
    }
}

#[test]
fn question_close_records_the_closure_and_the_decision_edge() {
    let mut repo = repo();
    let question = question("0001");
    let decision = decision("0002");
    repo.transaction("seed", "test", |repo| {
        repo.put(&question)?;
        repo.put(&decision)
    })
    .unwrap();
    let outcome = close(
        &question.id().clone(),
        Closure::Decision,
        Some(decision.id().clone()),
    )
    .run(&mut repo)
    .unwrap();
    let node = repo.get(question.id()).unwrap().unwrap();
    match node.data() {
        NodeData::Question(data) => {
            assert_eq!(data.closure, Some(Closure::Decision));
            assert_eq!(data.evidence.as_deref(), Some("closing evidence"));
        }
        _ => panic!("not a question"),
    }
    assert_eq!(node.links().len(), 1);
    assert_eq!(repo.incoming(decision.id()).unwrap()[0].name(), "closed-by");
    assert_eq!(outcome.changed, ["closed"]);
    let entry = repo.store().history().last().unwrap();
    assert_eq!(entry.why, format!("question close {}", question.id()));
    assert_eq!(entry.source, "closing evidence");
}

#[test]
fn closing_by_decision_without_a_decision_is_rejected() {
    let mut repo = repo();
    let question = question("0001");
    repo.transaction("seed", "test", |repo| repo.put(&question))
        .unwrap();
    assert!(
        close(question.id(), Closure::Decision, None)
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn closing_an_already_closed_question_is_rejected() {
    let mut repo = repo();
    let question = question("0001");
    repo.transaction("seed", "test", |repo| repo.put(&question))
        .unwrap();
    close(question.id(), Closure::Fact, None)
        .run(&mut repo)
        .unwrap();
    assert!(
        close(question.id(), Closure::Fact, None)
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn closing_without_evidence_is_rejected() {
    let mut repo = repo();
    let question = question("0001");
    repo.transaction("seed", "test", |repo| repo.put(&question))
        .unwrap();
    let mut request = close(question.id(), Closure::Fact, None);
    request.evidence = "  ".into();
    assert!(request.run(&mut repo).is_err());
}

#[test]
fn closing_a_non_question_is_rejected() {
    let mut repo = repo();
    let decision = decision("0002");
    repo.transaction("seed", "test", |repo| repo.put(&decision))
        .unwrap();
    assert!(
        close(decision.id(), Closure::Fact, None)
            .run(&mut repo)
            .is_err()
    );
}
