mod common;

use common::{fixture, need, stderr, stdout};

#[test]
fn edit_changes_title_and_free_attributes() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need")]);

    let out = fx.run(&[
        "edit",
        "n-0001",
        "--reason",
        "clarify",
        "--title",
        "new title",
        "--set",
        "owner=piko",
        "--append",
        "note=one",
        "--append",
        "note=two",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: title, free:owner, free:note"));
    assert!(stdout(&out).contains("missing: \nnext: \n"));

    let text = stdout(&fx.run(&["show", "--full", "n-0001"]));
    assert!(text.contains("new title"));
    assert!(text.contains("owner: piko"));
    assert!(text.contains("note: one\ntwo"));
}

#[test]
fn edit_moves_a_node_to_a_declared_scope() {
    let fx = fixture();
    std::fs::write(fx.root.join("gy.toml"), "[scopes.a]\n[scopes.b]\n").unwrap();
    fx.seed(&[need("0001", "a need")]);

    let out = fx.run(&["edit", "n-0001", "--reason", "move", "--set", "scope=b"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: scope"), "{}", stdout(&out));
    let text = stdout(&fx.run(&["show", "--full", "n-0001"]));
    assert!(text.contains("scope: b"), "{text}");

    let out = fx.run(&["edit", "n-0001", "--reason", "move", "--set", "scope=c"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn edit_names_the_declared_scopes() {
    let fx = fixture();
    std::fs::write(fx.root.join("gy.toml"), "[scopes.a]\n[scopes.b]\n").unwrap();
    fx.seed(&[need("0001", "a need")]);

    let out = fx.run(&["edit", "n-0001", "--reason", "move", "--set", "scope=nope"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("expected one of a, b"), "{message}");
}

#[test]
fn edit_help_shows_the_repeated_attribute_forms() {
    let fx = fixture();
    let help = fx.run(&["edit", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let text = stdout(&help);
    assert!(text.contains("--set A=B --set C=D"), "{text}");
    assert!(text.contains("--append A=B --append C=D"), "{text}");
}

#[test]
fn edit_prints_the_fifth_line_and_the_json_field() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need")]);

    let out = fx.run(&[
        "edit", "n-0001", "--reason", "clarify", "--title", "renamed",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("missing: \nnext: \n"), "{text}");
    assert!(text.ends_with("unresolved: \n"), "{text}");

    let json = fx.run(&[
        "--json", "edit", "n-0001", "--reason", "clarify", "--title", "again",
    ]);
    assert!(json.status.success(), "{}", stderr(&json));
    let value: serde_json::Value = serde_json::from_str(stdout(&json).trim()).unwrap();
    assert_eq!(value["unresolved"], serde_json::json!([]));
}

#[test]
fn edit_errors() {
    let fx = fixture();
    fx.seed(&[need("0002", "a need")]);

    let out = fx.run(&["edit", "n-0002", "--reason", " ", "--title", "x"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["edit", "n-0002", "--reason", "x", "--set", "status=done"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["edit", "n-0002", "--reason", "x"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["edit", "n-9999", "--reason", "x", "--title", "y"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["edit", "n-0002", "--reason", "x", "--set", "novalue"]);
    assert_eq!(out.status.code(), Some(2));
}
