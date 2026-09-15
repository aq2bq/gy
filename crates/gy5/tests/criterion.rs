mod common;

use common::{first_line, fixture, stderr, stdout};

#[test]
fn the_first_write_creates_the_ledger_and_resolves_one_scope() {
    let fx = fixture();
    assert!(!fx.ledger().exists());

    let out = fx.run(&["criterion", "add", "a criterion"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(fx.ledger().join("format").is_file());
}

#[test]
fn a_write_without_an_actor_does_not_create_the_ledger() {
    let fx = fixture();
    let out = fx.run_without_actor(&["criterion", "add", "a criterion"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(!fx.ledger().exists());
}

#[test]
fn criterion_add_satisfy_and_revoke() {
    let fx = fixture();
    let out = fx.run(&["criterion", "add", "a criterion"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: created"));
    assert!(stdout(&out).contains("next: criterion satisfy <AC> --evidence …"));
    let id = first_line(&out);

    let out = fx.run(&["criterion", "satisfy", &id, "--evidence", "verified"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: satisfied"));

    let out = fx.run(&["criterion", "satisfy", &id, "--evidence", "again"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&[
        "criterion",
        "satisfy",
        &id,
        "--evidence",
        "gone",
        "--revoke",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn criterion_json_and_actor() {
    let fx = fixture();
    let out = fx.run(&["--json", "criterion", "add", "a criterion"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["id"].is_string());
    assert_eq!(json["changed"][0], "created");

    let out = fx.run_without_actor(&["criterion", "add", "another"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("GY_ACTOR"));
}

#[test]
fn several_scopes_need_the_option_and_an_unknown_scope_is_an_error() {
    let fx = fixture();
    std::fs::write(fx.root.join("gy.toml"), "[scopes.a]\n[scopes.b]\n").unwrap();

    let out = fx.run(&["criterion", "add", "a criterion"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["--scope", "b", "criterion", "add", "a criterion"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let out = fx.run(&["--scope", "c", "criterion", "add", "a criterion"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("unknown scope"));
}
