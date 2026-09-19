mod common;

use common::{fixture, id_of, stderr, stdout};

#[test]
fn question_add_help_shows_the_repeated_form() {
    let fx = fixture();
    let out = fx.run(&["question", "add", "--help"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("--options A --options B"), "{text}");
}

#[test]
fn question_add_usage_shows_the_repeated_form() {
    let fx = fixture();
    let out = fx.run(&[
        "question",
        "add",
        "t",
        "--decider",
        "m",
        "--options",
        "A",
        "B",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("--options <A> --options <B>"),
        "{}",
        stderr(&out)
    );

    let help = fx.run(&["question", "add", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    assert!(
        stdout(&help).contains("--options <A> --options <B>"),
        "{}",
        stdout(&help)
    );
}

#[test]
fn question_add_then_close() {
    let fx = fixture();
    let out = fx.run(&[
        "question",
        "add",
        "a question",
        "--decider",
        "master",
        "--options",
        "a",
        "--options",
        "b",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: created, decider, options"));
    assert!(stdout(&out).contains("missing: a body (the ground for the options)"));
    let id = id_of(&out);

    let out = fx.run(&[
        "question",
        "close",
        &id,
        "--by",
        "fact",
        "--evidence",
        "observed",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: closed"));

    let out = fx.run(&["show", &id]);
    assert!(stdout(&out).contains("state: closed"));
}

#[test]
fn question_close_names_the_valid_by_values() {
    let fx = fixture();
    let id = id_of(&fx.run(&[
        "question",
        "add",
        "q",
        "--decider",
        "m",
        "--options",
        "a",
        "--options",
        "b",
    ]));

    let out = fx.run(&["question", "close", &id, "--by", "nope", "--evidence", "e"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("expected one of"), "{message}");
    for word in ["fact", "decision", "non-decision"] {
        assert!(message.contains(word), "{message}");
    }
}

#[test]
fn question_close_help_names_the_by_values() {
    let fx = fixture();
    let help = fx.run(&["question", "close", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let text = stdout(&help);
    for word in ["fact", "decision", "non-decision"] {
        assert!(text.contains(word), "{text}");
    }
}

#[test]
fn question_errors_and_json() {
    let fx = fixture();
    let out = fx.run(&[
        "--json",
        "question",
        "add",
        "q",
        "--decider",
        "m",
        "--options",
        "a",
        "--options",
        "b",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["id"].is_string());

    let out = fx.run(&[
        "question",
        "add",
        "q",
        "--decider",
        "m",
        "--options",
        "only",
    ]);
    assert_eq!(out.status.code(), Some(2));

    let id = id_of(&fx.run(&[
        "question",
        "add",
        "q",
        "--decider",
        "m",
        "--options",
        "a",
        "--options",
        "b",
    ]));
    let out = fx.run(&[
        "question",
        "close",
        &id,
        "--by",
        "decision",
        "--evidence",
        "decided",
    ]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run_without_actor(&[
        "question",
        "add",
        "q",
        "--decider",
        "m",
        "--options",
        "a",
        "--options",
        "b",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("GY_ACTOR"));
}
