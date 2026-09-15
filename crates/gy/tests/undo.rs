mod common;

use common::{first_line, fixture, stderr, stdout};

#[test]
fn undo_restores_the_previous_value() {
    let fx = fixture();
    let out = fx.run(&["criterion", "add", "a criterion"]);
    let id = first_line(&out);
    let out = fx.run(&["criterion", "satisfy", &id, "--evidence", "verified"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&fx.run(&["show", &id])).contains("satisfied: true"));

    let out = fx.run(&["undo", "--reason", "mistake"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: undone"));
    assert!(stdout(&out).contains("missing: \nnext: \n"));
    assert!(stdout(&fx.run(&["show", &id])).contains("satisfied: false"));
}

#[test]
fn undo_errors_without_a_reason_or_anything_to_undo() {
    let fx = fixture();
    let out = fx.run(&["undo", "--reason", "mistake"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["undo"]);
    assert_eq!(out.status.code(), Some(2));
}
