mod support;

use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn run(p: &Path, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(p)
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
fn summary(node: &Value) {
    let keys: Vec<_> = node
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        ["body_changed", "changed_attributes", "id", "scope", "type"]
    );
}

#[test]
fn body_file_results_match_cli_and_mcp_without_echoing_input() {
    let cli = support::baseline("test", None);
    let mcp = support::baseline("test", None);
    let body = "private body must not be returned\n".repeat(1000);
    for p in [cli.path(), mcp.path()] {
        fs::write(p.join("body.md"), &body).unwrap();
    }
    let args = [
        "set",
        "#1",
        "--body-file",
        "body.md",
        "--set",
        "custom=private attribute value",
        "--set",
        "body=attribute named body",
    ];
    let expected = run(cli.path(), &[&["node"][..], &args].concat());
    summary(&expected["node"]);
    assert_eq!(
        expected["node"]["changed_attributes"],
        json!(["body", "custom"])
    );
    assert_eq!(expected["node"]["body_changed"], true);
    assert!(expected.to_string().len() < 512);
    assert!(!expected.to_string().contains("private"));
    assert!(!expected.to_string().contains("attribute named body"));
    let mut child = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(mcp.path())
        .args(["mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "{}", json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"gy_node","arguments":{"args":args}}})).unwrap();
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["result"]["structuredContent"]["result"], expected);
    assert!(!String::from_utf8_lossy(&out.stdout).contains("private"));
    for p in [cli.path(), mcp.path()] {
        let saved = run(p, &["show", "#1"]);
        assert_eq!(saved["node"]["body"].as_str().unwrap().trim(), body.trim());
        assert_eq!(saved["node"]["attrs"]["custom"], "private attribute value");
        let unchanged = run(p, &[&["node"][..], &args].concat());
        assert_eq!(unchanged["node"]["changed_attributes"], json!([]));
        assert_eq!(unchanged["node"]["body_changed"], false);
        fs::write(p.join("body.md"), "replacement private body\n").unwrap();
        let changed = run(p, &["node", "set", "#1", "--body-file", "body.md"]);
        assert_eq!(changed["node"]["changed_attributes"], json!([]));
        assert_eq!(changed["node"]["body_changed"], true);
    }
    let human = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(cli.path())
        .args(["node", "set", "#1", "--body-file", "body.md"])
        .output()
        .unwrap();
    assert!(human.status.success());
    let text = String::from_utf8(human.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    assert!(!text.contains("private"));
}

#[test]
fn creation_satisfaction_and_links_return_only_changed_attribute_names() {
    let t = support::baseline("test", None);
    let p = t.path();
    let created = run(
        p,
        &["criterion", "add", "Private criterion", "--scope", "test"],
    );
    summary(&created["node"]);
    let saved = run(p, &["show", "AC-1"]);
    let names: Vec<_> = saved["node"]["attrs"].as_object().unwrap().keys().collect();
    assert_eq!(created["node"]["changed_attributes"], json!(names));
    assert!(!created.to_string().contains("Private criterion"));
    let satisfied = run(
        p,
        &[
            "criterion",
            "satisfy",
            "AC-1",
            "--evidence",
            "Private evidence",
        ],
    );
    assert_eq!(
        satisfied["node"]["changed_attributes"],
        json!(["evidence", "satisfied", "satisfied_at"])
    );
    assert!(!satisfied.to_string().contains("Private evidence"));
    let repeated = run(
        p,
        &[
            "criterion",
            "satisfy",
            "AC-1",
            "--evidence",
            "Other evidence",
        ],
    );
    assert_eq!(repeated["node"]["changed_attributes"], json!([]));
    let linked = run(p, &["link", "#1", "targets", "AC-1"]);
    summary(&linked["source"]);
    summary(&linked["target"]);
    assert_eq!(linked["source"]["changed_attributes"], json!(["targets"]));
    assert_eq!(
        linked["target"]["changed_attributes"],
        json!(["targeted-by"])
    );
    let repeated = run(p, &["link", "#1", "targets", "AC-1"]);
    assert_eq!(repeated["source"]["changed_attributes"], json!([]));
    assert_eq!(repeated["target"]["changed_attributes"], json!([]));
}

#[test]
fn decision_and_question_mutations_preserve_warning_results() {
    let t = support::baseline("test", None);
    let p = t.path();
    run(
        p,
        &[
            "question",
            "add",
            "Choose publication order",
            "--decider",
            "master",
            "--options",
            "arrival",
            "--options",
            "publication",
            "--scope",
            "test",
        ],
    );
    let decision = run(
        p,
        &[
            "decide",
            "Use publication order",
            "--scope-note",
            "Production deliveries",
            "--scope",
            "test",
        ],
    );
    summary(&decision["node"]);
    assert!(!decision["open_questions"].as_array().unwrap().is_empty());
    let question = run(
        p,
        &[
            "question",
            "add",
            "Choose publication order",
            "--decider",
            "master",
            "--options",
            "arrival",
            "--options",
            "publication",
            "--scope",
            "test",
            "--force",
        ],
    );
    summary(&question["node"]);
    assert!(!question["search_hits"].as_array().unwrap().is_empty());
}
