mod common;

use common::{decision, fixture, id_of, need, question, stderr, stdout};

#[test]
fn decide_creates_closes_and_links_one_relation() {
    let fx = fixture();
    let mut old = decision("0002", "an old decision");
    old.set_body("the changed part");
    fx.seed(&[question("0001", "a question"), old]);

    let out = fx.run(&[
        "decide",
        "a decision",
        "--scope-note",
        "applies at dawn",
        "--closes",
        "q-0001",
        "--relate",
        "narrows",
        "d-0002",
        "--mark",
        "changed part",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: created, closes, narrows"));
    assert!(stdout(&out).contains("missing: a body"));
    let id = id_of(&out);

    assert!(stdout(&fx.run(&["show", "q-0001"])).contains("state: closed"));
    let text = stdout(&fx.run(&["show", "--full", &id]));
    assert!(text.contains("decision_scope: applies at dawn"));
    assert!(text.contains("narrows d-0002"));
}

#[test]
fn decide_help_shows_the_repeated_closes_form() {
    let fx = fixture();
    let help = fx.run(&["decide", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    assert!(
        stdout(&help).contains("--closes A --closes B"),
        "{}",
        stdout(&help)
    );
}

#[test]
fn decide_relate_names_the_valid_relations() {
    let fx = fixture();
    fx.seed(&[decision("0003", "an old decision")]);

    let out = fx.run(&[
        "decide",
        "d",
        "--scope-note",
        "x",
        "--relate",
        "bogus",
        "d-0003",
    ]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("expected one of"), "{message}");
    for relation in gy_ledger::Relation::ALL {
        assert!(message.contains(relation.name()), "{message}");
    }
}

/// ac-57a8: `--relate` lists only the lineage relations decide accepts and
/// says which of them need a mark (d-d889).
#[test]
fn decide_help_names_only_the_lineage_relations() {
    let fx = fixture();
    let help = fx.run(&["decide", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let text = stdout(&help);
    let relate = text
        .lines()
        .find(|line| line.contains("--relate"))
        .unwrap_or_else(|| panic!("decide --help has no --relate line: {text}"));
    for relation in gy_ledger::Relation::lineage() {
        assert!(relate.contains(relation.name()), "{relate}");
    }
    assert!(
        relate.contains("narrows and supersedes also need --mark"),
        "{relate}"
    );
    for relation in gy_ledger::Relation::ALL {
        if gy_ledger::Relation::lineage().contains(&relation) {
            continue;
        }
        assert!(
            !relate.contains(relation.name()),
            "{} is not accepted by decide --relate, but the help names it: {relate}",
            relation.name()
        );
    }
}

#[test]
fn decide_errors_and_json() {
    let fx = fixture();
    fx.seed(&[decision("0003", "an old decision"), need("0004", "a need")]);

    let out = fx.run(&["decide", "d", "--scope-note", "x", "--mark", "m"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "decide",
        "d",
        "--scope-note",
        "x",
        "--relate",
        "narrows",
        "d-9999",
        "--mark",
        "m",
    ]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "decide",
        "d",
        "--scope-note",
        "x",
        "--relate",
        "targets",
        "d-0003",
    ]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "decide",
        "d",
        "--scope-note",
        "x",
        "--relate",
        "bogus",
        "d-0003",
    ]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "decide",
        "d",
        "--scope-note",
        "x",
        "--relate",
        "narrows",
        "d-0003",
        "--relate",
        "widens",
        "d-0003",
    ]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["decide", "d"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "--json",
        "decide",
        "d",
        "--scope-note",
        "x",
        "--serves",
        "n-0004",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["id"].is_string());
    assert_eq!(json["changed"][0], "created");
}

#[test]
fn show_without_full_names_the_passage_a_decision_retracted() {
    let fx = fixture();
    let mut old = decision("0002", "an old decision");
    old.set_body("## Decision\nthe old passage here\n");
    fx.seed(&[old, need("0003", "a need")]);

    let out = fx.run(&[
        "decide",
        "a narrowing decision",
        "--scope-note",
        "now",
        "--serves",
        "n-0003",
        "--relate",
        "narrows",
        "d-0002",
        "--mark",
        "the old passage here",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let newer = id_of(&out);

    let text = stdout(&fx.run(&["show", "d-0002"]));
    assert!(
        text.contains(&format!("[[retracted by {newer}: the old passage here]]")),
        "{text}"
    );
    assert!(
        text.contains(&format!("narrowed-by {newer} (the old passage here)")),
        "{text}"
    );

    let out = fx.run(&["--json", "show", "d-0002"]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json[0]["body"], "## Decision\nthe old passage here");
    assert!(
        json[0]["body_marked"]
            .as_str()
            .unwrap()
            .contains("[[retracted by"),
        "{json}"
    );
}

/// The writes the ledger holds, as the number of transactions recorded: a
/// refused write leaves it where it was (ac-c6d2).
fn writes(fx: &common::Fixture) -> usize {
    std::fs::read_to_string(fx.ledger().join("events.jsonl"))
        .map(|text| text.lines().count())
        .unwrap_or(0)
}

#[test]
fn decide_refuses_a_decision_that_serves_nothing() {
    let fx = fixture();
    let mut older = decision("0002", "an old decision");
    older.set_body("## Decision\nthe old passage here\n");
    fx.seed(&[older, need("0003", "a need")]);
    let before = writes(&fx);

    let out = fx.run(&["decide", "d", "--scope-note", "x"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("needs what it is for"), "{message}");
    assert!(message.contains("--serves <N|R|AC>"), "{message}");
    assert!(message.contains("--closes <Q>"), "{message}");
    assert!(message.contains("--relate"), "{message}");

    // A lineage edge to a decision that has nothing does not tie the new one.
    let out = fx.run(&[
        "decide",
        "d",
        "--scope-note",
        "x",
        "--relate",
        "narrows",
        "d-0002",
        "--mark",
        "the old passage here",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("needs what it is for"), "{out:?}");

    assert_eq!(writes(&fx), before, "a refused decide writes nothing");
    let listed = stdout(&fx.run(&["list", "--type", "Decision"]));
    assert_eq!(listed.lines().count(), 1, "{listed}");
}

#[test]
fn decide_serves_an_approved_requirement() {
    let fx = fixture();
    let ac = common::criterion("0001", "a criterion");
    fx.seed(&[ac]);
    let requirement = common::cover(&fx, "ac-0001");

    let out = fx.run(&[
        "decide",
        "we ship it",
        "--scope-note",
        "until the beta",
        "--serves",
        &requirement,
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("changed: created, serves"),
        "{}",
        stdout(&out)
    );
    let id = id_of(&out);

    let shown = stdout(&fx.run(&["show", "--full", &id]));
    assert!(shown.contains(&format!("serves {requirement}")), "{shown}");
}

#[test]
fn decide_refuses_a_serves_target_that_is_not_a_need_requirement_or_criterion() {
    let fx = fixture();
    fx.seed(&[decision("0001", "an old decision"), need("0002", "a need")]);

    let out = fx.run(&["decide", "d", "--scope-note", "x", "--serves", "d-0001"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("is not a need, a requirement, or a criterion"),
        "{out:?}"
    );

    let out = fx.run(&["decide", "d", "--scope-note", "x", "--serves", "n-9999"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("no node matches"), "{out:?}");
}

#[test]
fn show_names_what_a_decision_still_lacks_and_the_link_that_ties_it() {
    let fx = fixture();
    fx.seed(&[
        decision("0001", "a decision written before the rule"),
        need("0002", "a need"),
    ]);

    let shown = stdout(&fx.run(&["show", "d-0001"]));
    assert!(shown.contains("what it serves (serves)"), "{shown}");
}
