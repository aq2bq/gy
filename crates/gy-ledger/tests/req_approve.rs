use gy_ledger::{
    Actor, FormatVersion, MemoryStore, NodeData, NodeId, NodeKind, Operation, Repository,
    ReqApprove, RequirementState, Store,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn requirement(repo: &mut Repository<MemoryStore>) -> NodeId {
    let node = gy_ledger::Node::requirement(
        NodeId::from_hash(NodeKind::Requirement, "0001").unwrap(),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    let id = node.id().clone();
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    id
}

fn approve(id: &NodeId) -> ReqApprove {
    ReqApprove {
        id: id.clone(),
        design: "design:1".into(),
        heard_by: "master".into(),
        evidence: "heard it".into(),
    }
}

#[test]
fn req_approve_records_the_approval() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let outcome = approve(&id).run(&mut repo).unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.state(), Some(RequirementState::Approved));
    match node.data() {
        NodeData::Requirement(data) => {
            let approval = data.approval.as_ref().unwrap();
            assert_eq!(approval.heard_by, "master");
            assert_eq!(approval.design, "design:1");
            assert!(!approval.at.is_empty());
        }
        _ => panic!("not a requirement"),
    }
    assert_eq!(outcome.changed, ["approved"]);
    let entry = repo.store().history().last().unwrap();
    assert_eq!(entry.why, format!("req approve {id}"));
    assert_eq!(entry.source, "design:1");
}

#[test]
fn req_approve_needs_all_three_fields() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = approve(&id);
    request.design = "  ".into();
    assert!(request.run(&mut repo).is_err());
    assert_eq!(
        repo.get(&id).unwrap().unwrap().state(),
        Some(RequirementState::Filed)
    );
}

#[test]
fn req_approve_rejects_a_non_requirement() {
    let mut repo = repo();
    let id = NodeId::from_hash(NodeKind::Question, "0002").unwrap();
    let node = gy_ledger::Node::question(id.clone(), SCOPE, DATE, "a question").unwrap();
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    assert!(approve(&id).run(&mut repo).is_err());
}
