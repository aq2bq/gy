mod common;

use common::{fixture, id_of, stderr, stdout};
use gy_ledger::{Node, NodeId, NodeKind};

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        "a criterion",
    )
    .unwrap()
}

#[test]
fn need_add_then_close() {
    let fx = fixture();
    fx.seed(&[criterion("0001")]);

    let out = fx.run(&["need", "add", "a need", "--targets", "ac-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: created, targets"));
    assert!(stdout(&out).contains("missing: a body, a filed-as requirement"));
    let id = id_of(&out);

    let out = fx.run(&[
        "need",
        "close",
        &id,
        "--by",
        "fact",
        "--evidence",
        "resolved",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("changed: closed"));

    let out = fx.run(&["need", "close", &id, "--by", "fact", "--evidence", "again"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run(&["show", &id]);
    assert!(stdout(&out).contains("state: closed"));
}

#[test]
fn need_close_names_the_valid_by_values() {
    let fx = fixture();
    fx.seed(&[criterion("0001")]);
    let id = id_of(&fx.run(&["need", "add", "a need", "--targets", "ac-0001"]));

    let out = fx.run(&["need", "close", &id, "--by", "nope", "--evidence", "e"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    assert!(message.contains("expected one of"), "{message}");
    for word in ["fact", "external"] {
        assert!(message.contains(word), "{message}");
    }
}

#[test]
fn need_close_help_names_the_by_values() {
    let fx = fixture();
    let help = fx.run(&["need", "close", "--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let text = stdout(&help);
    for word in ["fact", "external"] {
        assert!(text.contains(word), "{text}");
    }
}

#[test]
fn need_add_usage_shows_the_repeated_form() {
    let fx = fixture();
    let out = fx.run(&["need", "add", "t", "--targets", "ac-0001", "ac-0002"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("--targets <AC> [--targets <AC>]..."),
        "{}",
        stderr(&out)
    );
}

#[test]
fn need_errors_and_json() {
    let fx = fixture();
    fx.seed(&[criterion("0001")]);

    let out = fx.run(&["--json", "need", "add", "a need", "--targets", "ac-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["id"].is_string());
    assert_eq!(json["changed"][0], "created");

    let out = fx.run(&["need", "add", "a need", "--targets", "ac-9999"]);
    assert_eq!(out.status.code(), Some(2));

    let out = fx.run_without_actor(&["need", "add", "a need", "--targets", "ac-0001"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("GY_ACTOR"));
}
