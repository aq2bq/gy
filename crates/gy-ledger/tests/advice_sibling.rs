//! n-5a63 / r-c83e: a satisfied criterion's next names only the unsatisfied
//! criteria of a need that also targets it, and reaches an unsatisfied one
//! behind a satisfied sibling.
use gy_ledger::{
    Actor, CriterionSatisfy, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind, Operation,
    Relation, Repository, RequirementState,
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

fn criterion(hash: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "a criterion").unwrap()
}

fn targets(need: &mut Node, criterion: &Node) {
    need.link(Link::new(need.id().clone(), Relation::Targets, criterion.id().clone()).unwrap());
}

/// An approved requirement that targets `criteria`, so the satisfy step itself
/// is what a following next names.
fn covering(criteria: &[&Node]) -> Node {
    let mut node = Node::requirement(
        id(NodeKind::Requirement, "0009"),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Approved,
    )
    .unwrap();
    for criterion in criteria {
        node.link(Link::new(node.id().clone(), Relation::Targets, criterion.id().clone()).unwrap());
    }
    node
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

#[test]
fn an_unrelated_need_s_criterion_is_not_pointed_at() {
    let mut repo = repo();
    let mut bearer = need("0001");
    let satisfied = criterion("0002");
    targets(&mut bearer, &satisfied);

    let mut stranger = need("0003");
    let elsewhere = criterion("0004");
    targets(&mut stranger, &elsewhere);
    let request = covering(&[&satisfied, &elsewhere]);
    seed(
        &mut repo,
        &[bearer, stranger, satisfied.clone(), elsewhere, request],
    );

    let outcome = CriterionSatisfy {
        id: satisfied.id().clone(),
        evidence: "verified".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    assert!(outcome.next.is_empty(), "{:?}", outcome.next);
}

#[test]
fn a_satisfied_sibling_does_not_hide_the_unsatisfied_one_behind_it() {
    let mut repo = repo();
    let mut bearer = need("0001");
    let first = criterion("0002");
    let middle = criterion("0003");
    let last = criterion("0004");
    for criterion in [&first, &middle, &last] {
        targets(&mut bearer, criterion);
    }
    let request = covering(&[&first, &middle, &last]);
    seed(
        &mut repo,
        &[bearer, first.clone(), middle.clone(), last.clone(), request],
    );
    let satisfied_middle = CriterionSatisfy {
        id: middle.id().clone(),
        evidence: "middle holds".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(
        satisfied_middle.next,
        [format!("criterion satisfy {} --evidence …", first.id())]
    );

    let outcome = CriterionSatisfy {
        id: first.id().clone(),
        evidence: "first holds".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(
        outcome.next,
        [format!("criterion satisfy {} --evidence …", last.id())]
    );
}

#[test]
fn no_unsatisfied_sibling_suggests_nothing_more() {
    let mut repo = repo();
    let mut bearer = need("0001");
    let only = criterion("0002");
    targets(&mut bearer, &only);
    let request = covering(&[&only]);
    seed(&mut repo, &[bearer, only.clone(), request]);

    let outcome = CriterionSatisfy {
        id: only.id().clone(),
        evidence: "verified".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    assert!(outcome.next.is_empty(), "{:?}", outcome.next);
}
