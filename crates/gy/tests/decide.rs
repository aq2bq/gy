mod common;

use common::{decision, first_line, fixture, question, stderr, stdout};

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
    assert!(stdout(&out).contains("missing: 本文"));
    let id = first_line(&out);

    assert!(stdout(&fx.run(&["show", "q-0001"])).contains("state: closed"));
    let text = stdout(&fx.run(&["show", "--full", &id]));
    assert!(text.contains("decision_scope: applies at dawn"));
    assert!(text.contains("narrows d-0002"));
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
