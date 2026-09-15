use std::collections::BTreeMap;

use gy_ledger::{
    Attributes, Closure, DecisionScope, Link, Node, NodeData, NodeId, NodeKind, Relation,
    RequirementState, bearer_count, free_attribute,
};

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

#[test]
fn five_kinds_build_and_a_gate_is_not_one() {
    assert_eq!(NodeKind::ALL.len(), 5);
    Node::need(id(NodeKind::Need, "0001"), "a need").unwrap();
    Node::question(id(NodeKind::Question, "0002"), "a question").unwrap();
    Node::decision(
        id(NodeKind::Decision, "0003"),
        "d",
        DecisionScope::recorded("applies in production").unwrap(),
    )
    .unwrap();
    Node::requirement(
        id(NodeKind::Requirement, "0004"),
        "r",
        RequirementState::Filed,
    )
    .unwrap();
    Node::criterion(id(NodeKind::Criterion, "0005"), "c").unwrap();
}

#[test]
fn an_empty_title_is_rejected() {
    assert!(Node::need(id(NodeKind::Need, "0001"), "   ").is_err());
}

#[test]
fn a_decision_needs_scope_or_the_migration_marker() {
    assert!(DecisionScope::recorded("   ").is_err());
    let scope = DecisionScope::migration_unrecorded();
    assert!(scope.is_unrecorded() && scope.text().is_empty());
    assert!(Node::decision(id(NodeKind::Decision, "0003"), "d", scope).is_ok());
}

#[test]
fn only_the_four_states_and_the_allowed_transitions() {
    use RequirementState::*;
    let mut requirement = Node::requirement(id(NodeKind::Requirement, "0004"), "r", Filed).unwrap();
    requirement.advance(Approved).unwrap();
    requirement.advance(Filed).unwrap();
    requirement.advance(Approved).unwrap();
    requirement.advance(Done).unwrap();
    assert_eq!(requirement.state(), Some(Done));
    assert!(Filed.advance(Done).is_err());
    assert!(Done.advance(Filed).is_err());
    assert!(Cancelled.advance(Approved).is_err());
    let mut cancelled = Node::requirement(id(NodeKind::Requirement, "0005"), "r", Filed).unwrap();
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

    let mut node = Node::need(id(NodeKind::Need, "0001"), "n").unwrap();
    node.set_free("owner", "team-a");
    assert_eq!(node.free("owner").map(String::as_str), Some("team-a"));
}

#[test]
fn a_question_closes_in_three_ways() {
    for closure in [Closure::Fact, Closure::Decision, Closure::NonDecision] {
        let mut question = Node::question(id(NodeKind::Question, "0002"), "q").unwrap();
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
    let mut first = Node::need(id(NodeKind::Need, "0001"), "n").unwrap();
    match first.data_mut() {
        NodeData::Need(data) => data.targets.push(criterion.clone()),
        _ => panic!("not a need"),
    }
    let second = Node::need(id(NodeKind::Need, "0003"), "n").unwrap();
    assert_eq!(bearer_count(&[first, second], &criterion), 1);
}
