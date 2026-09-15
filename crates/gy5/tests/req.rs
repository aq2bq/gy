mod common;

use common::{criterion, decision, first_line, fixture, need, stderr, stdout};

fn seed(fx: &common::Fixture) {
    let mut decision = decision("0002", "a decision");
    decision.set_body("scope");
    fx.seed(&[
        criterion("0001", "a criterion"),
        need("0003", "a need"),
        decision,
    ]);
}

#[test]
fn req_add_approve_done_with_the_ref_on_the_id_line() {
    let fx = fixture();
    seed(&fx);
    let out = fx.run(&[
        "req",
        "add",
        "a requirement",
        "--need",
        "n-0003",
        "--targets",
        "ac-0001",
        "--ref",
        "https://example/7",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("changed: created, needs"), "{text}");
    assert!(text.contains("(https://example/7)"), "{text}");
    let id = first_line(&out).split(' ').next().unwrap().to_string();

    let out = fx.run(&[
        "req",
        "approve",
        "https://example/7",
        "--design",
        "design:1",
        "--heard-by",
        "master",
        "--evidence",
        "heard it",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: approved"));

    let out = fx.run(&["req", "done", &id, "--evidence", "shipped"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: done"));
}

#[test]
fn req_revise_and_cancel() {
    let fx = fixture();
    seed(&fx);
    let out = fx.run(&["req", "add", "a requirement", "--need", "n-0003"]);
    let id = first_line(&out);

    let out = fx.run(&[
        "req",
        "approve",
        &id,
        "--design",
        "design:1",
        "--heard-by",
        "master",
        "--evidence",
        "e",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));

    let out = fx.run(&[
        "req",
        "revise",
        &id,
        "--reason",
        "changed",
        "--source",
        "conversation",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: revised"));

    let out = fx.run(&[
        "req",
        "approve",
        &id,
        "--design",
        "design:2",
        "--heard-by",
        "master",
        "--evidence",
        "e",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));

    let out = fx.run(&[
        "req",
        "cancel",
        &id,
        "--reason",
        "not needed",
        "--source",
        "conversation",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: cancelled"));
}

#[test]
fn req_errors_and_json() {
    let fx = fixture();
    seed(&fx);

    let out = fx.run(&["req", "add", "r"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "--json",
        "req",
        "add",
        "r",
        "--need",
        "n-0003",
        "--ref",
        "https://example/9",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["id"].is_string());
    assert_eq!(json["reference"], "https://example/9");

    let id = first_line(&fx.run(&["req", "add", "r", "--need", "n-0003"]));
    let out = fx.run(&["req", "done", &id, "--evidence", "x"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "req",
        "approve",
        "r-9999",
        "--design",
        "d",
        "--heard-by",
        "h",
        "--evidence",
        "e",
    ]);
    assert_eq!(out.status.code(), Some(2));
}
