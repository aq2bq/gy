//! n-f921: a satisfy needs an approved or done requirement that targets the
//! criterion; the output says the one next step. The rebase and share entries
//! are in `sync_rebase.rs`, which already has the two-copy harness.
use gy_ledger::{
    Actor, ClosedBy, CriterionSatisfy, Edit, FormatVersion, Link, MemoryStore, NeedClose, Node,
    NodeData, NodeId, NodeKind, Operation, Relation, Repository, RequirementState, Store, Undo,
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

fn criterion() -> Node {
    Node::criterion(id(NodeKind::Criterion, "0001"), SCOPE, DATE, "an ac").unwrap()
}

fn need() -> Node {
    Node::need(id(NodeKind::Need, "0002"), SCOPE, DATE, "a need").unwrap()
}

fn request(state: RequirementState) -> Node {
    Node::requirement(
        id(NodeKind::Requirement, "0003"),
        SCOPE,
        DATE,
        "a requirement",
        state,
    )
    .unwrap()
}

fn targets(node: &mut Node, criterion: &Node) {
    node.link(Link::new(node.id().clone(), Relation::Targets, criterion.id().clone()).unwrap());
}

fn satisfy(id: &NodeId, revoke: bool) -> CriterionSatisfy {
    CriterionSatisfy {
        id: id.clone(),
        evidence: "verified".into(),
        revoke,
    }
}

fn seed(repo: &mut Repository<MemoryStore>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn satisfied(node: &Node) -> bool {
    matches!(node.data(), NodeData::Criterion(data) if data.satisfied)
}

fn edit(id: &NodeId, title: Option<&str>, key: &str) -> Edit {
    Edit {
        id: id.clone(),
        reason: "clarify".into(),
        title: title.map(str::to_string),
        body: None,
        set: vec![(key.into(), "x".into())],
        append: Vec::new(),
    }
}

#[test]
fn a_satisfy_without_a_requirement_is_refused_and_writes_nothing() {
    let mut bare = repo();
    let ac = criterion();
    seed(&mut bare, std::slice::from_ref(&ac));
    let error = satisfy(ac.id(), false).run(&mut bare).unwrap_err();
    assert!(error.message.contains("req add"), "{}", error.message);
    assert!(!satisfied(&bare.get(ac.id()).unwrap().unwrap()));
    assert_eq!(bare.store().history().len(), 1);

    // A filed requirement that targets it is still not coverage.
    let mut repo = repo();
    let ac = criterion();
    let mut request = request(RequirementState::Filed);
    targets(&mut request, &ac);
    seed(&mut repo, &[ac.clone(), request.clone()]);
    let error = satisfy(ac.id(), false).run(&mut repo).unwrap_err();
    assert!(
        error
            .message
            .contains(&format!("req approve {}", request.id())),
        "{}",
        error.message
    );
}

#[test]
fn a_criterion_created_satisfied_without_coverage_is_refused() {
    let mut repo = repo();
    let mut ac = criterion();
    if let NodeData::Criterion(data) = ac.data_mut() {
        data.satisfied = true;
    }
    assert!(
        repo.transaction("seed", "test", |repo| repo.put(&ac))
            .is_err()
    );
    assert!(repo.all().unwrap().is_empty());
}

#[test]
fn a_closed_need_points_at_the_filed_requirement_to_approve() {
    let mut repo = repo();
    let ac = criterion();
    let mut need = need();
    targets(&mut need, &ac);
    let mut request = request(RequirementState::Filed);
    targets(&mut request, &ac);
    seed(&mut repo, &[ac.clone(), need.clone(), request.clone()]);
    let outcome = NeedClose {
        id: need.id().clone(),
        by: ClosedBy::Fact,
        evidence: "closed".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(
        outcome.next,
        [format!(
            "req approve {} --design … --heard-by … --evidence …",
            request.id()
        )]
    );
}

/// A pre-rule ledger: a satisfied criterion with no coverage, seeded without
/// the gate. Unrelated edits pass, but a revoke-then-undo cannot restore it.
#[test]
fn the_rule_leaves_an_existing_ledger_alone() {
    let mut store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut met = criterion();
    if let NodeData::Criterion(data) = met.data_mut() {
        data.satisfied = true;
    }
    let other = need();
    store
        .transaction(|store| {
            for node in [&met, &other] {
                let key = node.id().to_string();
                store.stage(key.clone(), serde_json::to_vec(node).unwrap());
                store.record(&key, "created", "seed", "test");
            }
            Ok(())
        })
        .unwrap();

    let mut repo = Repository::new(store);
    edit(other.id(), Some("renamed"), "note")
        .run(&mut repo)
        .unwrap();
    edit(met.id(), None, "note").run(&mut repo).unwrap();
    assert!(satisfied(&repo.get(met.id()).unwrap().unwrap()));
    satisfy(met.id(), true).run(&mut repo).unwrap();
    assert!(
        Undo {
            reason: "mistake".into(),
        }
        .run(&mut repo)
        .is_err(),
        "the undo restored a satisfy with no coverage"
    );
}
