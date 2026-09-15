use gy_ledger::{
    Actor, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Operation, Repository,
    ReqApprove, ReqCancel, ReqDone, RequirementState, Store,
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
    let node = Node::requirement(
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

fn cancel(id: &NodeId) -> ReqCancel {
    ReqCancel {
        id: id.clone(),
        reason: "not needed".into(),
        source: "conversation".into(),
    }
}

#[test]
fn req_cancel_records_the_cancellation() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let outcome = cancel(&id).run(&mut repo).unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.state(), Some(RequirementState::Cancelled));
    match node.data() {
        NodeData::Requirement(data) => {
            assert_eq!(data.cancellation.as_ref().unwrap().reason, "not needed");
        }
        _ => panic!("not a requirement"),
    }
    assert_eq!(outcome.changed, ["cancelled"]);
    let entry = repo.store().history().last().unwrap();
    assert_eq!(entry.why, format!("req cancel {id}"));
    assert_eq!(entry.source, "conversation");
}

#[test]
fn req_cancel_rejects_a_done_requirement_and_empty_fields() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = cancel(&id);
    request.reason = "  ".into();
    assert!(request.run(&mut repo).is_err());

    ReqApprove {
        id: id.clone(),
        design: "design:1".into(),
        heard_by: "master".into(),
        evidence: "heard it".into(),
    }
    .run(&mut repo)
    .unwrap();
    ReqDone {
        id: id.clone(),
        evidence: "shipped".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert!(cancel(&id).run(&mut repo).is_err());
}
