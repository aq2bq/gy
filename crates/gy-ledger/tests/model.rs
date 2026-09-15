use std::collections::BTreeMap;

use gy_ledger::{
    Attributes, Closure, DecisionScope, Link, Node, NodeData, NodeId, NodeKind, Relation,
    RequirementState, bearer_count, free_attribute,
};

const DATE: &str = "2026-09-15";
const SCOPE: &str = "a";

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn need(hash: &str, title: &str) -> Node {
    Node::need(id(NodeKind::Need, hash), SCOPE, DATE, title).unwrap()
}

fn question(hash: &str, title: &str) -> Node {
    Node::question(id(NodeKind::Question, hash), SCOPE, DATE, title).unwrap()
}

#[test]
fn five_kinds_build_and_a_gate_is_not_one() {
    assert_eq!(NodeKind::ALL.len(), 5);
    need("0001", "a need");
    question("0002", "a question");
    Node::decision(
        id(NodeKind::Decision, "0003"),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("applies in production").unwrap(),
    )
    .unwrap();
    Node::requirement(
        id(NodeKind::Requirement, "0004"),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    Node::criterion(id(NodeKind::Criterion, "0005"), SCOPE, DATE, "a criterion").unwrap();
}

#[test]
fn an_empty_title_scope_or_bad_date_is_rejected() {
    assert!(Node::need(id(NodeKind::Need, "0001"), SCOPE, DATE, "   ").is_err());
    assert!(Node::need(id(NodeKind::Need, "0001"), "  ", DATE, "a need").is_err());
    assert!(Node::need(id(NodeKind::Need, "0001"), SCOPE, "2026/09/15", "a need").is_err());
}

#[test]
fn a_decision_needs_scope_or_the_migration_marker() {
    assert!(DecisionScope::recorded("   ").is_err());
    let scope = DecisionScope::migration_unrecorded();
    assert!(scope.is_unrecorded() && scope.text().is_empty());
    assert!(Node::decision(id(NodeKind::Decision, "0003"), SCOPE, DATE, "d", scope).is_ok());
}

#[test]
fn only_the_four_states_and_the_allowed_transitions() {
    use RequirementState::*;
    let mut requirement =
        Node::requirement(id(NodeKind::Requirement, "0004"), SCOPE, DATE, "r", Filed).unwrap();
    requirement.advance(Approved).unwrap();
    requirement.advance(Filed).unwrap();
    requirement.advance(Approved).unwrap();
    requirement.advance(Done).unwrap();
    assert_eq!(requirement.state(), Some(Done));
    assert!(Filed.advance(Done).is_err());
    assert!(Done.advance(Filed).is_err());
    assert!(Cancelled.advance(Approved).is_err());
    let mut cancelled =
        Node::requirement(id(NodeKind::Requirement, "0005"), SCOPE, DATE, "r", Filed).unwrap();
    cancelled.advance(Cancelled).unwrap();
    assert!(requirement.advance(Cancelled).is_err());
}

#[test]
fn a_link_sets_both_directions() {
    let need = id(NodeKind::Need, "0001");
    let criterion = id(NodeKind::Criterion, "0002");
    let link = Link::new(need.clone(), Relation::Targets, criterion.clone()).unwrap();
    assert_eq!(
        (
            &link.forward().from,
            &link.forward().to,
            link.forward().name()
        ),
        (&need, &criterion, "targets")
    );
    assert_eq!(
        (
            &link.reverse().from,
            &link.reverse().to,
            link.reverse().name()
        ),
        (&criterion, &need, "targeted-by")
    );
}

#[test]
fn allowed_kind_pairs_build() {
    let need = id(NodeKind::Need, "0001");
    let criterion = id(NodeKind::Criterion, "0002");
    let question = id(NodeKind::Question, "0003");
    let decision = id(NodeKind::Decision, "0004");
    assert!(Link::new(need, Relation::Targets, criterion).is_ok());
    assert!(Link::new(question, Relation::Closes, decision).is_ok());
}

#[test]
fn a_disallowed_kind_pair_is_rejected() {
    let need = id(NodeKind::Need, "0001");
    let criterion = id(NodeKind::Criterion, "0002");
    let decision = id(NodeKind::Decision, "0004");
    assert!(Link::new(criterion.clone(), Relation::Closes, need.clone()).is_err());
    assert!(Link::new(need, Relation::Narrows, decision).is_err());
}

#[test]
fn the_inverse_of_a_relation_is_derived() {
    assert_eq!(Relation::Closes.name(), "closes");
    assert_eq!(Relation::Closes.inverse(), "closed-by");
    assert_eq!(Relation::Targets.inverse(), "targeted-by");
    assert_eq!(Relation::SpawnedBy.inverse(), "spawns");
    assert_eq!(Relation::FiledAs.inverse(), "files");
}

#[test]
fn free_attributes_pass_through_one_boundary() {
    let mut bag = BTreeMap::new();
    bag.insert("region".to_string(), "east".to_string());
    let mut raw = Attributes::new();
    raw.insert("attributes".to_string(), bag);
    assert_eq!(
        free_attribute(&raw, "region").map(String::as_str),
        Some("east")
    );
    assert_eq!(free_attribute(&raw, "missing"), None);

    let mut node = need("0001", "n");
    node.set_free("owner", "team-a");
    assert_eq!(node.free("owner").map(String::as_str), Some("team-a"));
    assert_eq!((node.scope(), node.created()), (SCOPE, DATE));
}

#[test]
fn a_question_closes_in_three_ways() {
    for closure in [Closure::Fact, Closure::Decision, Closure::NonDecision] {
        let mut question = question("0002", "q");
        if let NodeData::Question(data) = question.data_mut() {
            data.closure = Some(closure);
        }
        match question.data() {
            NodeData::Question(data) => assert_eq!(data.closure, Some(closure)),
            _ => panic!("not a question"),
        }
    }
}

#[test]
fn bearer_count_is_derived_not_stored() {
    let criterion = id(NodeKind::Criterion, "0002");
    let mut first = need("0001", "n");
    first.link(Link::new(first.id().clone(), Relation::Targets, criterion.clone()).unwrap());
    let second = need("0003", "n");
    assert_eq!(bearer_count(&[first, second], &criterion), 1);
}
