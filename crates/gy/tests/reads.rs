mod common;

use common::{criterion, decision, fixture, need, question, stderr, stdout};
use gy_ledger::{Link, Node, NodeId, NodeKind, Relation};
use serde_json::Value;

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

/// ac-88ca: `show --json` carries every edge in both directions, `created`,
/// and the writer who created the node, with or without `--full`; the text
/// output keeps showing edges only with `--full` (d-bbab).
#[test]
fn show_json_carries_both_edge_directions_created_and_the_creator() {
    let fx = fixture();
    let criterion = criterion("0010", "a criterion");
    let mut need = need("0011", "a need");
    need.link(Link::new(need.id().clone(), Relation::Targets, criterion.id().clone()).unwrap());
    let question = question("0012", "a question");
    need.link(Link::new(need.id().clone(), Relation::WaitsOn, question.id().clone()).unwrap());
    let mut older = decision("0013", "an older decision");
    older.set_body("the old passage");
    let mut newer = decision("0014", "a newer decision");
    newer.link(
        Link::new(newer.id().clone(), Relation::Narrows, older.id().clone())
            .unwrap()
            .with_mark(Some("old passage".into())),
    );
    let mut wider = decision("0015", "a wider decision");
    wider.link(Link::new(wider.id().clone(), Relation::Widens, older.id().clone()).unwrap());
    let mut serving = decision("0016", "a decision that serves");
    serving.link(Link::new(serving.id().clone(), Relation::Serves, need.id().clone()).unwrap());
    fx.seed(&[criterion, need, question, older, newer, wider, serving]);

    let out = fx.run(&["--json", "show", "n-0011", "q-0012", "d-0013"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let nodes = json.as_array().unwrap();

    let edges = |node: &Value| -> Vec<(String, String, Option<String>)> {
        node["edges"]
            .as_array()
            .unwrap_or_else(|| panic!("{} has no edges: {node}", node["id"]))
            .iter()
            .map(|edge| {
                (
                    edge["name"].as_str().unwrap().to_string(),
                    edge["to"].as_str().unwrap().to_string(),
                    edge["mark"].as_str().map(str::to_string),
                )
            })
            .collect()
    };

    let need = nodes.iter().find(|node| node["id"] == "n-0011").unwrap();
    assert_eq!(need["created"], "2026-09-15T00:00:00Z");
    assert_eq!(need["actor"], "piko");
    let need_edges = edges(need);
    assert!(
        need_edges.contains(&("targets".into(), "ac-0010".into(), None)),
        "{need_edges:?}"
    );
    assert!(
        need_edges.contains(&("waits-on".into(), "q-0012".into(), None)),
        "{need_edges:?}"
    );
    assert!(
        need_edges.contains(&("served-by".into(), "d-0016".into(), None)),
        "{need_edges:?}"
    );

    let question = nodes.iter().find(|node| node["id"] == "q-0012").unwrap();
    assert_eq!(
        edges(question),
        [("awaited-by".into(), "n-0011".into(), None)]
    );

    let older = nodes.iter().find(|node| node["id"] == "d-0013").unwrap();
    let older_edges = edges(older);
    assert!(
        older_edges.contains(&(
            "narrowed-by".into(),
            "d-0014".into(),
            Some("old passage".into())
        )),
        "{older_edges:?}"
    );
    assert!(
        older_edges.contains(&("widened-by".into(), "d-0015".into(), None)),
        "{older_edges:?}"
    );

    let full = fx.run(&["show", "--full", "d-0013"]);
    assert!(full.status.success(), "{}", stderr(&full));
    let text = stdout(&full);
    assert!(text.contains("narrowed-by d-0014 (old passage)"), "{text}");
    assert!(text.contains("widened-by d-0015"), "{text}");
    assert!(text.contains("created: 2026-09-15"), "{text}");

    let plain = fx.run(&["show", "d-0013"]);
    assert!(plain.status.success(), "{}", stderr(&plain));
    let text = stdout(&plain);
    assert!(!text.contains("widened-by d-0015"), "{text}");
    assert!(!text.contains("created:"), "{text}");
}

#[test]
fn show_json_without_full_keeps_the_text_short() {
    let fx = fixture();
    let mut node = need("0001", "a need");
    node.set_body("the body");
    fx.seed(&[node]);

    let out = fx.run(&["show", "n-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!stdout(&out).contains("created:"), "{}", stdout(&out));

    let out = fx.run(&["--json", "show", "n-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json[0]["created"], "2026-09-15T00:00:00Z");
    assert_eq!(json[0]["actor"], "piko");
    assert_eq!(json[0]["edges"], serde_json::json!([]));
}
