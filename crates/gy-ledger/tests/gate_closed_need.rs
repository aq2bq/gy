//! n-d5a2, d-ee34: a closed need may not take a new filed-as requirement, on
//! an ordinary write, in `req add`, and on an undo; a need that already filed a
//! requirement before it closed keeps it.
use gy_ledger::link::Link;
use gy_ledger::{
    Actor, ClosedBy, Edit, FormatVersion, MemoryStore, NeedClose, Node, NodeData, NodeId, NodeKind,
    Operation, Relation, Repository, ReqAdd, Undo, now,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}
fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}
fn need(hash: &str) -> Node {
    Node::need(id(NodeKind::Need, hash), SCOPE, DATE, "a need").unwrap()
}
fn requirement(hash: &str) -> Node {
    Node::requirement(
        id(NodeKind::Requirement, hash),
        SCOPE,
        DATE,
        "a requirement",
        gy_ledger::RequirementState::Filed,
    )
    .unwrap()
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
fn close(repo: &mut Repository<MemoryStore>, id: &NodeId) {
    NeedClose {
        id: id.clone(),
        by: ClosedBy::Fact,
        evidence: "done".to_string(),
    }
    .run(repo)
    .unwrap();
}
fn files(from: &NodeId, to: &NodeId, remove: bool) -> Link {
    Link {
        from: from.clone(),
        relation: Relation::FiledAs,
        to: to.clone(),
        mark: None,
        remove,
    }
}
fn add(need: &NodeId) -> ReqAdd {
    ReqAdd {
        scope: SCOPE.to_string(),
        title: "a requirement".to_string(),
        needs: vec![need.clone()],
        relies_on: Vec::new(),
        targets: Vec::new(),
        reference: None,
        body: None,
    }
}
fn edge_count(repo: &Repository<MemoryStore>, from: &NodeId) -> usize {
    repo.get(from)
        .unwrap()
        .unwrap()
        .links()
        .iter()
        .filter(|edge| !edge.reversed && edge.label == Relation::FiledAs)
        .count()
}
fn seq(repo: &Repository<MemoryStore>) -> u64 {
    now(repo, None).unwrap().seq
}
fn refused_closed(result: &gy_ledger::Result<gy_ledger::Outcome<NodeId>>, id: &NodeId) -> bool {
    result
        .as_ref()
        .is_err_and(|error| error.message == format!("{id} is closed"))
}

#[test]
fn req_add_refuses_a_closed_need_and_writes_nothing() {
    let mut repo = repo();
    let n = need("0001");
    seed(&mut repo, &[&n]);
    close(&mut repo, n.id());
    let before = seq(&repo);
    let refused = add(n.id()).run(&mut repo);
    assert!(refused_closed(&refused, n.id()), "{refused:?}");
    assert_eq!(seq(&repo), before);
}

#[test]
fn link_filing_a_requirement_onto_a_closed_need_is_refused() {
    let mut repo = repo();
    let n = need("0001");
    let r = requirement("0002");
    seed(&mut repo, &[&n, &r]);
    close(&mut repo, n.id());
    let before = seq(&repo);
    let refused = files(n.id(), r.id(), false).run(&mut repo);
    assert!(refused_closed(&refused, n.id()), "{refused:?}");
    assert_eq!(seq(&repo), before);
    assert_eq!(edge_count(&repo, n.id()), 0);
}

#[test]
fn a_need_that_filed_a_requirement_before_it_closed_closes() {
    let mut repo = repo();
    let n = need("0001");
    let r = requirement("0002");
    seed(&mut repo, &[&n, &r]);
    files(n.id(), r.id(), false).run(&mut repo).unwrap();
    close(&mut repo, n.id());
    let node = repo.get(n.id()).unwrap().unwrap();
    assert!(matches!(node.data(), NodeData::Need(data) if data.closed.is_some()));
    assert_eq!(edge_count(&repo, n.id()), 1);
}

#[test]
fn an_existing_filed_as_on_a_closed_need_passes_an_edit() {
    let mut repo = repo();
    let n = need("0001");
    let r = requirement("0002");
    seed(&mut repo, &[&n, &r]);
    files(n.id(), r.id(), false).run(&mut repo).unwrap();
    close(&mut repo, n.id());
    Edit {
        id: n.id().clone(),
        reason: "tidy".to_string(),
        title: Some("renamed".to_string()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(edge_count(&repo, n.id()), 1);
}

#[test]
fn an_undo_cannot_put_a_filed_as_onto_a_closed_need() {
    let mut repo = repo();
    let n = need("0001");
    let r = requirement("0002");
    seed(&mut repo, &[&n, &r]);
    files(n.id(), r.id(), false).run(&mut repo).unwrap();
    close(&mut repo, n.id());
    files(n.id(), r.id(), true).run(&mut repo).unwrap();
    assert_eq!(edge_count(&repo, n.id()), 0);

    // The undo would put the filed-as back, onto the closed need, so the gate
    // refuses it and the edge stays out.
    assert!(
        Undo {
            reason: "mistake".to_string(),
        }
        .run(&mut repo)
        .is_err()
    );
    assert_eq!(edge_count(&repo, n.id()), 0);
}
