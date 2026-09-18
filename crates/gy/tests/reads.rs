mod common;

use common::{fixture, stderr, stdout};
use gy_ledger::{Node, NodeId, NodeKind};

fn need(hash: &str, title: &str) -> Node {
    Node::need(
        NodeId::from_hash(NodeKind::Need, hash).unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        title,
    )
    .unwrap()
}

#[test]
fn show_prints_a_node_and_its_json() {
    let fx = fixture();
    let mut node = need("0001", "a need");
    node.set_body("the body");
    fx.seed(&[node]);

    let out = fx.run(&["show", "n-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("a need"));
    assert!(text.contains("state: open"));
    assert!(text.contains("the body"));

    let out = fx.run(&["--json", "show", "n-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json[0]["kind"], "Need");
    assert_eq!(json[0]["id"], "n-0001");
}

#[test]
fn list_filters_by_type_and_status() {
    let fx = fixture();
    let criterion = Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, "0002").unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        "a criterion",
    )
    .unwrap();
    fx.seed(&[need("0001", "a need"), criterion]);

    let out = fx.run(&["list", "--type", "need"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("a need"));
    assert!(!text.contains("a criterion"));

    let out = fx.run(&["list", "--status", "open"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("a need"));
}

#[test]
fn next_and_handover_report_the_open_work() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need")]);

    let out = fx.run(&["next"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("a need"));

    let out = fx.run(&["handover"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("ready needs: 1"));
    assert!(text.contains("in progress: 0"));
}

#[test]
fn reads_do_not_need_an_actor() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need")]);
    let out = fx.run_without_actor(&["show", "n-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn a_missing_repository_or_ledger_is_an_error() {
    let fx = fixture();
    std::fs::remove_file(fx.root.join("gy.toml")).unwrap();
    let out = fx.run(&["show", "n-0001"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("no gy.toml"));

    let fx = fixture();
    let out = fx.run(&["handover"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("no ledger"));
}

#[test]
fn an_invalid_read_exits_two_with_a_json_error() {
    let fx = fixture();
    fx.seed(&[need("0001", "a need")]);
    let out = fx.run(&["--json", "show", "n-9999"]);
    assert_eq!(out.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["code"], 2);
    assert!(
        json["message"]
            .as_str()
            .unwrap()
            .contains("no node matches")
    );
}
