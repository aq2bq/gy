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
