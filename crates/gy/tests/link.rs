mod common;

use common::{criterion, decision, fixture, id_of, need, question, stderr, stdout};

#[test]
fn link_adds_then_removes_an_edge() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need"), criterion("0002", "a criterion")]);

    let out = fx.run(&["link", "n-0001", "targets", "ac-0002"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: linked"));
    assert!(stdout(&out).contains("missing: \nnext: \n"));

    let out = fx.run(&["link", "n-0001", "targets", "ac-0002"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["link", "n-0001", "targets", "ac-0002", "--remove"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: unlinked"));
}

#[test]
fn link_rejects_closes_and_an_unknown_relation() {
    let fx = fixture();
    fx.seed(&[
        question("0003", "a question"),
        decision("0004", "a decision"),
    ]);
    let out = fx.run(&["link", "q-0003", "closes", "d-0004"]);
    assert_eq!(out.status.code(), Some(2));

    fx.seed(&[need("0005", "a need"), criterion("0006", "a criterion")]);
    let out = fx.run(&["link", "n-0005", "bogus", "ac-0006"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn link_names_the_valid_relations() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need"), criterion("0002", "a criterion")]);

    let out = fx.run(&["link", "n-0001", "bogus", "ac-0002"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("expected one of"), "{message}");
    for relation in gy_ledger::Relation::ALL {
        assert!(message.contains(relation.name()), "{message}");
    }
}

#[test]
fn link_help_names_the_relations() {
    let fx = fixture();
    let help = fx.run(&["link", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let text = stdout(&help);
    for relation in gy_ledger::Relation::ALL {
        assert!(text.contains(relation.name()), "{text}");
    }
}

/// A decision carrying the edge it holds, the way an earlier write left it.
fn serving(mut decision: gy_ledger::Node, to: &gy_ledger::Node) -> gy_ledger::Node {
    decision.link(
        gy_ledger::Link::new(
            decision.id().clone(),
            gy_ledger::Relation::Serves,
            to.id().clone(),
        )
        .unwrap(),
    );
    decision
}

/// A requirement, the way `req add` files one.
fn requirement_node(hash: &str, title: &str) -> gy_ledger::Node {
    gy_ledger::Node::requirement(
        common::node_id(gy_ledger::NodeKind::Requirement, hash),
        "a",
        common::DATE,
        title,
        gy_ledger::RequirementState::Filed,
    )
    .unwrap()
}

fn writes(fx: &common::Fixture) -> usize {
    std::fs::read_to_string(fx.ledger().join("events.jsonl"))
        .map(|text| text.lines().count())
        .unwrap_or(0)
}

fn serving_nothing(fx: &common::Fixture) -> usize {
    let text = stdout(&fx.run(&["handover"]));
    text.lines()
        .find_map(|line| line.strip_prefix("decisions serving nothing: "))
        .map_or(0, |count| count.trim().parse().unwrap())
}

#[test]
fn link_ties_a_decision_written_before_the_rule() {
    let fx = fixture();
    let ac = criterion("0001", "a criterion");
    fx.seed(&[decision("0002", "an orphan"), ac]);
    assert_eq!(serving_nothing(&fx), 1, "the orphan is counted");

    let out = fx.run(&["link", "d-0002", "serves", "ac-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: linked"));
    assert_eq!(serving_nothing(&fx), 0, "the count goes down");
}

#[test]
fn link_removes_a_tie_when_another_one_remains() {
    let fx = fixture();
    let first = criterion("0001", "a criterion");
    let second = criterion("0002", "another criterion");
    let mut decision = decision("0003", "a decision");
    decision.link(
        gy_ledger::Link::new(
            decision.id().clone(),
            gy_ledger::Relation::Serves,
            first.id().clone(),
        )
        .unwrap(),
    );
    fx.seed(&[first.clone(), second.clone(), serving(decision, &second)]);

    let out = fx.run(&["link", "d-0003", "serves", "ac-0001", "--remove"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(serving_nothing(&fx), 0);
}

#[test]
fn link_removes_the_only_serves_edge_when_the_decision_closes_a_question() {
    let fx = fixture();
    let ac = criterion("0001", "a criterion");
    fx.seed(&[ac]);
    let closed = id_of(&fx.run(&[
        "question",
        "add",
        "a question",
        "--decider",
        "someone",
        "--options",
        "one",
        "--options",
        "two",
    ]));
    let decision = id_of(&fx.run(&[
        "decide",
        "a decision",
        "--scope-note",
        "a scope",
        "--serves",
        "ac-0001",
        "--closes",
        &closed,
    ]));

    // The closed question still ties it, so its only serves edge may go.
    let out = fx.run(&["link", &decision, "serves", "ac-0001", "--remove"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(serving_nothing(&fx), 0);
}

#[test]
fn link_refuses_to_remove_the_last_tie_of_a_decision() {
    let fx = fixture();
    let ac = criterion("0001", "a criterion");
    fx.seed(&[serving(decision("0002", "a decision"), &ac), ac]);
    let before = writes(&fx);

    let out = fx.run(&["link", "d-0002", "serves", "ac-0001", "--remove"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("would serve nothing"), "{message}");
    assert!(message.contains("link d-0002 serves <N|R|AC>"), "{message}");
    assert_eq!(writes(&fx), before, "a refused link writes nothing");
    assert_eq!(serving_nothing(&fx), 0);
}

#[test]
fn link_refuses_to_remove_the_last_tie_seen_from_the_other_side() {
    let fx = fixture();
    let decision = decision("0002", "a decision");
    let mut requirement = requirement_node("0003", "a requirement");
    requirement.link(
        gy_ledger::Link::new(
            requirement.id().clone(),
            gy_ledger::Relation::ReliesOn,
            decision.id().clone(),
        )
        .unwrap(),
    );
    fx.seed(&[decision, requirement]);

    // The decision holds no serves edge: a requirement's relies-on ties it.
    let out = fx.run(&["link", "r-0003", "relies-on", "d-0002", "--remove"]);
    assert_eq!(out.status.code(), Some(2), "{:?}", stdout(&out));
    assert!(
        stderr(&out).contains("d-0002 would serve nothing"),
        "{out:?}"
    );
}

#[test]
fn link_refuses_to_remove_the_last_tie_a_need_holds() {
    let fx = fixture();
    let decision = decision("0002", "a decision");
    let mut called = need("0004", "a need");
    called.link(
        gy_ledger::Link::new(
            called.id().clone(),
            gy_ledger::Relation::SpawnedBy,
            decision.id().clone(),
        )
        .unwrap(),
    );
    fx.seed(&[decision, called]);

    let out = fx.run(&["link", "n-0004", "spawned-by", "d-0002", "--remove"]);
    assert_eq!(out.status.code(), Some(2), "{:?}", stdout(&out));
    assert!(
        stderr(&out).contains("d-0002 would serve nothing"),
        "{out:?}"
    );
}
