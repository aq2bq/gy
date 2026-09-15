mod common;

use common::{criterion, decision, fixture, need, question, stderr, stdout};

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
