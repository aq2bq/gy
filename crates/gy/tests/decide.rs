mod common;

use common::{decision, fixture, id_of, question, stderr, stdout};

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

#[test]
fn decide_help_names_the_relations() {
    let fx = fixture();
    let help = fx.run(&["decide", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let text = stdout(&help);
    for relation in gy_ledger::Relation::ALL {
        assert!(text.contains(relation.name()), "{text}");
    }
}

#[test]
fn decide_errors_and_json() {
    let fx = fixture();
    fx.seed(&[decision("0003", "an old decision")]);

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

    let out = fx.run(&["--json", "decide", "d", "--scope-note", "x"]);
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
    fx.seed(&[old]);

    let out = fx.run(&[
        "decide",
        "a narrowing decision",
        "--scope-note",
        "now",
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
