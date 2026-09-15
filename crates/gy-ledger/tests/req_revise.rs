use gy_ledger::{
    Actor, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Operation, Repository,
    ReqApprove, ReqRevise, RequirementState, Store,
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

fn approve(id: &NodeId, repo: &mut Repository<MemoryStore>) {
    ReqApprove {
        id: id.clone(),
        design: "design:1".into(),
        heard_by: "master".into(),
        evidence: "heard it".into(),
    }
    .run(repo)
    .unwrap();
}

fn revise(id: &NodeId) -> ReqRevise {
    ReqRevise {
        id: id.clone(),
        reason: "scope changed".into(),
        source: "conversation".into(),
    }
}

#[test]
fn req_revise_returns_to_filed_and_keeps_the_record() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    approve(&id, &mut repo);
    let outcome = revise(&id).run(&mut repo).unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.state(), Some(RequirementState::Filed));
    match node.data() {
        NodeData::Requirement(data) => {
            assert_eq!(data.revisions.len(), 1);
            assert_eq!(data.revisions[0].reason, "scope changed");
            assert_eq!(data.revisions[0].source, "conversation");
        }
        _ => panic!("not a requirement"),
    }
    assert_eq!(outcome.changed, ["revised"]);
    let entry = repo.store().history().last().unwrap();
    assert_eq!(entry.why, format!("req revise {id}"));
    assert_eq!(entry.source, "conversation");
}

#[test]
fn req_revise_needs_approval_a_reason_and_a_source() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    assert!(revise(&id).run(&mut repo).is_err());
    approve(&id, &mut repo);
    let mut request = revise(&id);
    request.reason = "  ".into();
    assert!(request.run(&mut repo).is_err());
}
