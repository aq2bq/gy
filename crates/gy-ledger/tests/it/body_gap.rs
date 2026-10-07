//! n-00d3: the edit lead follows an empty body, not the wording of the gap.
use gy_ledger::{
    Actor, FormatVersion, MemoryStore, NeedAdd, Node, NodeId, NodeKind, Operation, Repository,
    ReqAdd, Store,
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

fn seed<S: Store>(repo: &mut Repository<S>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn the_edit_lead_follows_an_empty_body_not_the_wording_of_the_gap() {
    let mut repo = repo();
    let ac = Node::criterion(id(NodeKind::Criterion, "0001"), SCOPE, DATE, "an AC").unwrap();
    let ac_id = ac.id().clone();
    let need = Node::need(id(NodeKind::Need, "0002"), SCOPE, DATE, "a need").unwrap();
    let need_id = need.id().clone();
    seed(&mut repo, &[ac, need]);

    // An empty body in a state that reports the gap: the edit leads.
    let added = NeedAdd {
        scope: SCOPE.into(),
        title: "a need".into(),
        targets: vec![ac_id],
        spawned_by: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    assert!(!added.missing.is_empty());
    assert!(added.next[0].starts_with("edit "), "{:?}", added.next);

    // An empty body in a state that does not report a body gap: no edit, even
    // though the node still has other gaps.
    let requirement = ReqAdd {
        scope: SCOPE.into(),
        title: "a requirement".into(),
        needs: vec![need_id],
        relies_on: Vec::new(),
        targets: Vec::new(),
        reference: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    assert!(!requirement.missing.is_empty());
    assert!(
        requirement.next.iter().all(|cmd| !cmd.starts_with("edit ")),
        "{:?}",
        requirement.next
    );
}
