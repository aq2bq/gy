use gy_ledger::{
    Actor, ClosedBy, DecisionScope, FormatVersion, MemoryStore, NeedClose, Node, NodeData, NodeId,
    NodeKind, Operation, Ref, Repository, ReqAdd, RequirementState,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

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

fn requirement(hash: &str, state: RequirementState, reference: &str) -> Node {
    let mut node = Node::requirement(
        NodeId::from_hash(NodeKind::Requirement, hash).unwrap(),
        SCOPE,
        DATE,
        "a requirement",
        state,
    )
    .unwrap();
    if let NodeData::Requirement(data) = node.data_mut() {
        data.reference = Some(Ref(reference.into()));
    }
    node
}

fn seed(repo: &mut Repository<MemoryStore>, nodes: &[&Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn add(needs: Vec<NodeId>, reference: Option<Ref>) -> ReqAdd {
    ReqAdd {
        scope: SCOPE.into(),
        title: "a requirement".into(),
        needs,
        relies_on: vec![],
        targets: vec![],
        reference,
    }
}

#[test]
fn req_add_creates_a_requirement_and_filed_as_edges() {
    let mut repo = repo();
    let one = need("0001");
    let two = need("0002");
    seed(&mut repo, &[&one, &two]);
    let outcome = add(
        vec![one.id().clone(), two.id().clone()],
        Some(Ref("tracker:6027".into())),
    )
    .run(&mut repo)
    .unwrap();
    let id = outcome.id.clone().unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.state(), Some(RequirementState::Filed));
    let incoming = repo.incoming(&id).unwrap();
    assert_eq!(incoming.len(), 2);
    assert!(incoming.iter().all(|edge| edge.name() == "files"));
    assert_eq!(outcome.changed, ["created", "needs"]);
}

#[test]
fn req_add_rejects_an_open_requirement_with_the_same_reference() {
    let mut repo = repo();
    let one = need("0001");
    let open = requirement("0003", RequirementState::Filed, "tracker:6027");
    seed(&mut repo, &[&one, &open]);
    assert!(
        add(vec![one.id().clone()], Some(Ref("tracker:6027".into())))
            .run(&mut repo)
            .is_err()
    );
}

#[test]
fn req_add_allows_the_same_reference_once_the_other_is_done() {
    let mut repo = repo();
    let need = need("0001");
    let done = requirement("0003", RequirementState::Done, "tracker:6027");
    seed(&mut repo, &[&need, &done]);
    assert!(
        add(vec![need.id().clone()], Some(Ref("tracker:6027".into())))
            .run(&mut repo)
            .is_ok()
    );
}

#[test]
fn req_add_rejects_no_needs_a_closed_need_or_a_wrong_kind() {
    let mut repo = repo();
    assert!(add(vec![], None).run(&mut repo).is_err());

    let need = need("0001");
    let decision = Node::decision(
        NodeId::from_hash(NodeKind::Decision, "0002").unwrap(),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap();
    seed(&mut repo, &[&need, &decision]);
    assert!(
        add(vec![decision.id().clone()], None)
            .run(&mut repo)
            .is_err()
    );

    NeedClose {
        id: need.id().clone(),
        by: ClosedBy::Fact,
        evidence: "done".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert!(add(vec![need.id().clone()], None).run(&mut repo).is_err());
}
