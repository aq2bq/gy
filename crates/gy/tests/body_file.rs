//! `--body-file` on the four add commands writes the body in the same
//! transaction (n-c1d1, r-c241): one write, where add plus edit was two. A
//! missing path stops before the node exists.
mod common;

use common::{fixture, id_of, stderr, stdout};
use serde_json::Value;

/// The last sequence the ledger holds; 0 while the ledger is not there yet.
fn last_seq(fx: &common::Fixture) -> u64 {
    let out = fx.run(&["list", "--since", "0", "--json"]);
    if !out.status.success() {
        return 0;
    }
    let rows: Vec<Value> = serde_json::from_str(&stdout(&out)).expect("rows");
    rows.iter()
        .filter_map(|row| row["seq"].as_u64())
        .max()
        .unwrap_or(0)
}

#[test]
fn a_body_file_lands_in_the_same_write() {
    let fx = fixture();
    let body = fx.root.join("body.txt");
    std::fs::write(&body, "the observed shape\n").expect("the body file");
    let path = body.to_str().expect("a path");

    let create = |args: &[&str]| {
        let seq = last_seq(&fx);
        let out = fx.run(args);
        assert!(out.status.success(), "{}", stderr(&out));
        assert_eq!(
            last_seq(&fx),
            seq + 1,
            "{args:?} wrote more than one transaction"
        );
        id_of(&out)
    };

    let criterion = create(&["criterion", "add", "measures one", "--body-file", path]);
    let need = create(&[
        "need",
        "add",
        "the need",
        "--targets",
        &criterion,
        "--body-file",
        path,
    ]);
    let question = create(&[
        "question",
        "add",
        "a question",
        "--decider",
        "master",
        "--options",
        "one",
        "--options",
        "two",
        "--body-file",
        path,
    ]);
    let requirement = create(&[
        "req",
        "add",
        "a requirement",
        "--need",
        &need,
        "--body-file",
        path,
    ]);

    // The body is on each node, and the advice no longer asks for it.
    for id in [&criterion, &need, &question, &requirement] {
        let out = fx.run(&["show", id, "--json"]);
        assert!(
            stdout(&out).contains("the observed shape"),
            "show {id} lacks the body"
        );
    }
}

#[test]
fn a_missing_body_file_creates_nothing() {
    let fx = fixture();
    let out = fx.run(&["criterion", "add", "measures one"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let seq = last_seq(&fx);

    let out = fx.run(&[
        "criterion",
        "add",
        "measures two",
        "--body-file",
        "/no/such/file",
    ]);
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert!(stderr(&out).contains("/no/such/file"), "{}", stderr(&out));
    assert_eq!(last_seq(&fx), seq, "a write happened anyway");
}
