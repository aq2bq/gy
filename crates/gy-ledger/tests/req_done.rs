use gy_ledger::{
    Actor, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Operation, Repository,
    ReqApprove, ReqDone, RequirementState, Store,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

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

#[test]
fn req_done_records_the_completion() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    approve(&id, &mut repo);
    let outcome = ReqDone {
        id: id.clone(),
        evidence: "shipped".into(),
    }
    .run(&mut repo)
    .unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.state(), Some(RequirementState::Done));
    match node.data() {
        NodeData::Requirement(data) => {
            assert_eq!(data.completion.as_ref().unwrap().evidence, "shipped");
            let at = &data.completion.as_ref().unwrap().at;
            assert!(at.len() == 20 && at.ends_with('Z'), "{at}");
        }
        _ => panic!("not a requirement"),
    }
    assert_eq!(outcome.changed, ["done"]);
    let entry = repo.store().history().last().unwrap();
    assert_eq!(entry.why, format!("req done {id}"));
    assert_eq!(entry.source, "shipped");
}

#[test]
fn req_done_needs_approval_and_evidence() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    assert!(
        ReqDone {
            id: id.clone(),
            evidence: "shipped".into()
        }
        .run(&mut repo)
        .is_err()
    );
    approve(&id, &mut repo);
    assert!(
        ReqDone {
            id: id.clone(),
            evidence: "  ".into()
        }
        .run(&mut repo)
        .is_err()
    );
}
