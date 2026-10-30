mod common;

use common::{fixture, id_of, stderr, stdout};

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
    let id = id_of(&out);
    assert!(
        stdout(&out).contains(&format!(
            "next: edit {id} --body-file … --reason …, req add \"<title>\" --need <N> --targets {id}"
        )),
        "{}",
        stdout(&out)
    );

    // An approved requirement that targets it is what admits the satisfy (n-f921).
    common::cover(&fx, &id);

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

/// ac-b885: a criterion only closed needs aimed at reads as withdrawn, with the
/// close's evidence as its reason; the targets edge stays, and the gap leaves
/// handover's warning and list's unsatisfied (d-ed67).
#[test]
fn a_criterion_only_closed_needs_aimed_at_is_withdrawn() {
    let fx = fixture();
    let ac = id_of(&fx.run(&["criterion", "add", "an abandoned criterion"]));
    let need = id_of(&fx.run(&["need", "add", "a need", "--targets", &ac]));

    let closed = fx.run(&[
        "need",
        "close",
        &need,
        "--by",
        "fact",
        "--evidence",
        "we no longer pursue this",
    ]);
    assert!(closed.status.success(), "{}", stderr(&closed));
    assert!(
        stdout(&closed).contains(&format!("unmet criterion {ac}")),
        "need close still names the unmet criterion: {}",
        stdout(&closed)
    );

    let shown = fx.run(&["show", &ac]);
    assert!(shown.status.success(), "{}", stderr(&shown));
    let text = stdout(&shown);
    assert!(text.contains("satisfied: false"), "{text}");
    assert!(
        text.contains(&format!(
            "withdrawn-by {need} (fact: we no longer pursue this)"
        )),
        "{text}"
    );

    let full = fx.run(&["show", "--full", &need]);
    assert!(full.status.success(), "{}", stderr(&full));
    assert!(
        stdout(&full).contains(&format!("targets {ac}")),
        "{}",
        stdout(&full)
    );

    let json = fx.run(&["--json", "show", &ac]);
    assert!(json.status.success(), "{}", stderr(&json));
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value[0]["withdrawn_by"][0]["need"], need);
    assert_eq!(value[0]["withdrawn_by"][0]["by"], "Fact");
    assert_eq!(
        value[0]["withdrawn_by"][0]["evidence"],
        "we no longer pursue this"
    );

    let handover = fx.run(&["handover"]);
    assert!(handover.status.success(), "{}", stderr(&handover));
    assert!(
        !stdout(&handover).contains("criteria unmet with every need closed"),
        "{}",
        stdout(&handover)
    );

    let unsatisfied = fx.run(&["list", "--type", "criterion", "--status", "unsatisfied"]);
    assert!(unsatisfied.status.success(), "{}", stderr(&unsatisfied));
    assert!(
        !stdout(&unsatisfied).contains(&ac),
        "{}",
        stdout(&unsatisfied)
    );

    let listed = fx.run(&["list", "--type", "criterion"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    assert!(stdout(&listed).contains("withdrawn"), "{}", stdout(&listed));
}

/// ac-b885 (4): one open need still aims at the criterion, so it is not
/// withdrawn.
#[test]
fn a_criterion_an_open_need_still_aims_at_is_not_withdrawn() {
    let fx = fixture();
    let ac = id_of(&fx.run(&["criterion", "add", "a live criterion"]));
    let closed = id_of(&fx.run(&["need", "add", "a closed need", "--targets", &ac]));
    fx.run(&["need", "add", "an open need", "--targets", &ac]);
    fx.run(&[
        "need",
        "close",
        &closed,
        "--by",
        "external",
        "--evidence",
        "tracked elsewhere",
    ]);

    let shown = fx.run(&["show", &ac]);
    assert!(shown.status.success(), "{}", stderr(&shown));
    let text = stdout(&shown);
    assert!(!text.contains("withdrawn-by"), "{text}");

    let unsatisfied = fx.run(&["list", "--type", "criterion", "--status", "unsatisfied"]);
    assert!(unsatisfied.status.success(), "{}", stderr(&unsatisfied));
    assert!(
        stdout(&unsatisfied).contains(&ac),
        "{}",
        stdout(&unsatisfied)
    );
}
