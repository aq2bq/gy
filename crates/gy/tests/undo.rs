mod common;

use common::{fixture, id_of, stderr, stdout};

#[test]
fn undo_restores_the_previous_value() {
    let fx = fixture();
    let out = fx.run(&["criterion", "add", "a criterion"]);
    let id = id_of(&out);
    // An approved requirement that targets it admits the satisfy and its redo
    // (n-f921).
    common::cover(&fx, &id);

    let out = fx.run(&["criterion", "satisfy", &id, "--evidence", "verified"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&fx.run(&["show", &id])).contains("satisfied: true"));

    let out = fx.run(&["undo", "--reason", "mistake"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: undone"));
    assert!(
        stdout(&out).contains("next: undo again"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&fx.run(&["show", &id])).contains("satisfied: false"));

    let out = fx.run(&["undo", "--reason", "again"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("next: this was a redo"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&fx.run(&["show", &id])).contains("satisfied: true"));
}

#[test]
fn undo_errors_without_a_reason_or_anything_to_undo() {
    let fx = fixture();
    let out = fx.run(&["undo", "--reason", "mistake"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["undo"]);
    assert_eq!(out.status.code(), Some(2));
}
