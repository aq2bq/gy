//! The one judgement of what a decision is for (d-753c, r-0a63): a decision is
//! bound by serving a need, a requirement, or a criterion, by closing a
//! question, by a requirement that relies on it, by a need it spawned, or by a
//! lineage edge that reaches a decision with one. decide, link, handover and
//! show all read this, so the copies cannot drift.
use gy_ledger::{
    Actor, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind, Relation,
    Repository, Store, advice_for, decision_bound,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
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

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn decision(hash: &str) -> Node {
    Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("a scope").unwrap(),
    )
    .unwrap()
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

fn criterion(hash: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "a criterion").unwrap()
}

fn question(hash: &str) -> Node {
    Node::question(id(NodeKind::Question, hash), SCOPE, DATE, "a question").unwrap()
}

/// The decision's own edge to what it is for.
fn serves(mut decision: Node, target: &Node) -> Node {
    decision.link(Link::new(decision.id().clone(), Relation::Serves, target.id().clone()).unwrap());
    decision
}

/// A lineage edge from `newer` to `older`, the way decide records one.
fn lineage(mut newer: Node, relation: Relation, older: &Node) -> Node {
    newer.link(Link::new(newer.id().clone(), relation, older.id().clone()).unwrap());
    newer
}

#[test]
fn a_decision_that_serves_a_need_requirement_or_criterion_is_bound() {
    for target in [need("0002"), requirement("0003"), criterion("0004")] {
        let decision = serves(decision("0001"), &target);
        let all = vec![decision.clone(), target];
        assert!(decision_bound(&decision, &all));
    }
}

#[test]
fn the_other_side_of_an_edge_binds_the_decision_too() {
    let decision = decision("0001");

    let mut closing = question("0002");
    closing.link(
        Link::new(
            closing.id().clone(),
            Relation::Closes,
            decision.id().clone(),
        )
        .unwrap(),
    );
    assert!(
        decision_bound(&decision, &[decision.clone(), closing]),
        "a question closed by the decision binds it"
    );

    let mut relying = requirement("0003");
    relying.link(
        Link::new(
            relying.id().clone(),
            Relation::ReliesOn,
            decision.id().clone(),
        )
        .unwrap(),
    );
    assert!(
        decision_bound(&decision, &[decision.clone(), relying]),
        "a requirement that relies on the decision binds it"
    );

    let mut spawning = need("0004");
    spawning.link(
        Link::new(
            spawning.id().clone(),
            Relation::SpawnedBy,
            decision.id().clone(),
        )
        .unwrap(),
    );
    assert!(
        decision_bound(&decision, &[decision.clone(), spawning]),
        "a need spawned by the decision binds it"
    );
}

#[test]
fn a_decision_with_nothing_it_is_for_is_not_bound() {
    let lone = decision("0001");
    let older = decision("0002");
    let need = need("0003");
    assert!(!decision_bound(&lone, &[lone.clone(), older, need]));
}

#[test]
fn lineage_reaches_the_decision_that_has_what_it_is_for() {
    let need = need("0001");
    let oldest = serves(decision("0002"), &need);
    let middle = lineage(decision("0003"), Relation::Narrows, &oldest);
    let newest = lineage(decision("0004"), Relation::Completes, &middle);
    let all = vec![need, oldest, middle.clone(), newest.clone()];
    assert!(decision_bound(&middle, &all));
    assert!(decision_bound(&newest, &all));
}

#[test]
fn lineage_to_a_decision_with_nothing_does_not_bind() {
    let older = decision("0002");
    let newer = lineage(decision("0001"), Relation::Supersedes, &older);
    assert!(!decision_bound(&newer, &[newer.clone(), older]));
}

#[test]
fn a_cycle_of_lineage_ends_and_binds_nothing() {
    let first = lineage(decision("0001"), Relation::Narrows, &decision("0002"));
    let second = lineage(decision("0002"), Relation::Widens, &decision("0001"));
    let all = vec![first.clone(), second.clone()];
    assert!(!decision_bound(&first, &all));
    assert!(!decision_bound(&second, &all));
}

#[test]
fn the_advice_names_what_a_decision_serves_and_the_link_that_ties_it() {
    let mut repo = repo();
    let mut lone = decision("0001");
    lone.set_body("what was decided");
    let need = need("0002");
    seed(&mut repo, &[lone.clone(), need]);

    let (missing, next) = advice_for(&repo, &lone).unwrap();
    assert_eq!(missing, ["what it serves (serves)"]);
    assert_eq!(next, ["link d-0001 serves <N|R|AC>"]);
}
