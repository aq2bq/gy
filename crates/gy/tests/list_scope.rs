//! r-dfbd: scoped lists keep the same rows and order in every CLI form.
mod common;

use common::{DATE, fixture, node_id, stderr, stdout};
use gy_ledger::{DecisionScope, Link, Node, NodeKind, Relation, RequirementState};
use serde_json::Value;
use std::collections::BTreeMap;

fn seeded() -> common::Fixture {
    let fx = fixture();
    std::fs::write(
        fx.root.join("gy.toml"),
        "[scopes.a]\n[scopes.b]\n[scopes.c]\n",
    )
    .unwrap();
    let mut nodes = Vec::new();
    for (i, scope) in ["a", "b", "c"].iter().enumerate() {
        let hash = format!("{:04x}", i + 1);
        let ac = Node::criterion(
            node_id(NodeKind::Criterion, &hash),
            scope,
            DATE,
            "needle criterion",
        )
        .unwrap();
        let mut need =
            Node::need(node_id(NodeKind::Need, &hash), scope, DATE, "needle need").unwrap();
        need.link(Link::new(need.id().clone(), Relation::Targets, ac.id().clone()).unwrap());
        let question = Node::question(
            node_id(NodeKind::Question, &hash),
            scope,
            DATE,
            "other question",
        )
        .unwrap();
        let mut decision = Node::decision(
            node_id(NodeKind::Decision, &hash),
            scope,
            DATE,
            "needle decision",
            DecisionScope::recorded("applies here").unwrap(),
        )
        .unwrap();
        decision
            .link(Link::new(decision.id().clone(), Relation::Serves, need.id().clone()).unwrap());
        let request = Node::requirement(
            node_id(NodeKind::Requirement, &hash),
            scope,
            DATE,
            "other requirement",
            RequirementState::Filed,
        )
        .unwrap();
        need.link(Link::new(need.id().clone(), Relation::FiledAs, request.id().clone()).unwrap());
        nodes.extend([ac, need, question, decision, request]);
    }
    fx.seed(&nodes);
    fx
}

fn run(fx: &common::Fixture, args: &[&str]) -> String {
    let out = fx.run(args);
    assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    stdout(&out)
}

fn json(fx: &common::Fixture, args: &[&str]) -> Value {
    let mut args = args.to_vec();
    args.push("--json");
    serde_json::from_str(&run(fx, &args)).unwrap()
}

/// Compare complete JSON objects and complete text lines, preserving order.
fn compare(fx: &common::Fixture, args: &[&str], scope: &str, scopes: &BTreeMap<String, String>) {
    let all = json(fx, args);
    let rows = all.as_array().unwrap();
    let text = run(fx, args);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), rows.len());
    let mut expected = Vec::new();
    let mut expected_text = String::new();
    for (row, line) in rows.iter().zip(lines) {
        let id = row.get("node").unwrap_or(&row["id"]).as_str().unwrap();
        if scopes.get(id).is_some_and(|name| name == scope) {
            expected.push(row.clone());
            expected_text.push_str(line);
            expected_text.push('\n');
        }
    }
    let mut scoped = args.to_vec();
    scoped.extend(["--scope", scope]);
    assert_eq!(json(fx, &scoped), Value::Array(expected), "{scoped:?}");
    assert_eq!(run(fx, &scoped), expected_text, "{scoped:?}");
}

#[test]
fn three_scopes_match_unscoped_rows_for_every_filter_and_combination() {
    let fx = seeded();
    let rows = json(&fx, &["list"]);
    assert_eq!(rows.as_array().unwrap().len(), 15);
    let mut scopes = BTreeMap::new();
    for row in rows.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let shown = json(&fx, &["show", "--full", id]);
        assert_eq!(shown[0]["scope"], row["scope"]);
        scopes.insert(
            id.to_string(),
            shown[0]["scope"].as_str().unwrap().to_string(),
        );
    }
    for (i, scope) in ["a", "b", "c"].iter().enumerate() {
        let target = format!("ac-{:04x}", i + 1);
        let filters = [
            vec!["--type", "need"],
            vec!["--status", "open"],
            vec!["--targets", &target],
            vec!["--grep", "needle"],
        ];
        for mask in 0..16 {
            let mut args = vec!["list"];
            for (bit, filter) in filters.iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    args.extend(filter);
                }
            }
            compare(&fx, &args, scope, &scopes);
        }
        for kind in ["need", "criterion", "question", "decision", "requirement"] {
            compare(&fx, &["list", "--type", kind], scope, &scopes);
        }
        for args in [
            vec!["list", "--since", "0"],
            vec!["list", "--since", "1"],
            vec!["list", "--actor", "piko"],
            vec!["list", "--actor", "absent"],
            vec!["list", "--since", "0", "--actor", "piko"],
            vec![
                "list",
                "--since",
                "0",
                "--actor",
                "piko",
                "--type",
                "need",
                "--status",
                "open",
                "--targets",
                &target,
                "--grep",
                "needle",
            ],
        ] {
            compare(&fx, &args, scope, &scopes);
        }
    }
}

#[test]
fn unknown_scopes_use_the_write_diagnostic_in_text_and_json() {
    let fx = seeded();
    let write = fx.run(&["--scope", "missing", "criterion", "add", "a criterion"]);
    assert!(!write.status.success());
    let message = stderr(&write);
    assert_eq!(
        message.trim(),
        "unknown scope missing; expected one of a, b, c"
    );
    for flags in [vec![], vec!["--type", "need"], vec!["--since", "0"]] {
        let mut args = vec!["list", "--scope", "missing"];
        args.extend(flags);
        let text = fx.run(&args);
        assert!(!text.status.success());
        assert!(text.stdout.is_empty());
        assert_eq!(stderr(&text), message);
        args.push("--json");
        let out = fx.run(&args);
        assert!(!out.status.success());
        assert!(out.stderr.is_empty());
        let error: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            error,
            serde_json::json!({"code": 2, "message": message.trim()})
        );
    }
    let all = fx.run(&["list", "--scope", "all"]);
    assert!(
        !all.status.success(),
        "CLI all is not a reserved scope name"
    );
    // With one declared scope, omitting --scope still lists every stored node.
    std::fs::write(fx.root.join("gy.toml"), "[scopes.a]\n").unwrap();
    assert_eq!(json(&fx, &["list"]).as_array().unwrap().len(), 15);
}

#[test]
fn scoped_history_omits_retired_nodes_and_writes_without_one_node() {
    let fx = fixture();
    std::fs::write(
        fx.root.join("gy.toml"),
        "[scopes.a]\n[scopes.b]\n[scopes.c]\n",
    )
    .unwrap();
    let nodes: Vec<Node> = ["a", "b", "c"]
        .iter()
        .enumerate()
        .map(|(i, scope)| {
            let mut node = Node::need(
                node_id(NodeKind::Need, &format!("{:04x}", i + 1)),
                scope,
                DATE,
                "closed need",
            )
            .unwrap();
            if let gy_ledger::NodeData::Need(data) = node.data_mut() {
                data.closed = Some(gy_ledger::Closed {
                    by: gy_ledger::ClosedBy::Fact,
                    evidence: "finished".into(),
                });
            }
            node
        })
        .collect();
    fx.seed(&nodes);
    let retired = ["n-0001", "n-0002"];
    let mut args = vec!["retire"];
    args.extend(retired);
    args.extend(["--reason", "retire two scopes together"]);
    run(&fx, &args);
    assert_eq!(
        json(&fx, &["list", "--scope", "a", "--since", "0"]),
        serde_json::json!([])
    );
    let unscoped = json(&fx, &["list", "--since", "0"]);
    assert!(
        unscoped
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["node"] == "" && row["what"].as_str().unwrap().contains("n-0002"))
    );
    args.push("--revoke");
    run(&fx, &args);
    run(&fx, &["scope", "rename", "a", "renamed"]);
    run(
        &fx,
        &[
            "edit",
            "n-0002",
            "--set",
            "scope=renamed",
            "--reason",
            "move the need",
        ],
    );
    let rows = json(&fx, &["list"]);
    let scopes = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["id"].as_str().unwrap().to_string(),
                row["scope"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    for scope in ["renamed", "b", "c"] {
        compare(&fx, &["list", "--since", "0"], scope, &scopes);
        compare(&fx, &["list", "--actor", "piko"], scope, &scopes);
    }
    let unscoped = json(&fx, &["list", "--since", "0"]);
    assert_eq!(
        unscoped
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["node"] == "")
            .count(),
        3
    );
    let moved = json(&fx, &["list", "--scope", "renamed", "--since", "0"]);
    assert_eq!(
        moved
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["node"] == "n-0002")
            .count(),
        2
    );
}
