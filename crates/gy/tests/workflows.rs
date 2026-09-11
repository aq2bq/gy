use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};
use tempfile::TempDir;

fn invoke(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(dir)
        .arg("--json")
        .args(args)
        .output()
        .unwrap()
}
fn run(dir: &Path, args: &[&str]) -> Value {
    let out = invoke(dir, args);
    assert!(
        out.status.success(),
        "{args:?}\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
fn reject(dir: &Path, args: &[&str], code: i32) -> String {
    let out = invoke(dir, args);
    assert_eq!(
        out.status.code(),
        Some(code),
        "{args:?}\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stderr).into()
}
fn repo() -> TempDir {
    let t = tempfile::tempdir().unwrap();
    run(t.path(), &["init", "a", "--parent-issue", "6000"]);
    t
}
fn add_ac(p: &Path) {
    run(
        p,
        &["criterion", "add", "Acceptance criterion", "--scope", "a"],
    );
}
fn add_q(p: &Path) {
    run(
        p,
        &[
            "question",
            "add",
            "How should deliveries be ordered?",
            "--decider",
            "master",
            "--options",
            "Publication order",
            "--options",
            "Arrival order",
            "--scope",
            "a",
        ],
    );
}
fn add_d(p: &Path) {
    run(
        p,
        &[
            "decide",
            "Order deliveries by publication time",
            "--scope-note",
            "Production workers only",
            "--scope",
            "a",
        ],
    );
}
fn edit_node(p: &Path, relative: &str, mutate: impl FnOnce(&mut gy_core::Node)) {
    let path = p.join("docs/ledger").join(relative);
    let mut n = gy_core::Node::parse(&fs::read_to_string(&path).unwrap(), path.clone()).unwrap();
    mutate(&mut n);
    fs::write(path, n.markdown().unwrap()).unwrap();
}

#[test]
fn init_preserves_agents_and_discovers_from_subdirectories() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("AGENTS.md"), "existing instruction").unwrap();
    run(t.path(), &["init", "a"]);
    run(t.path(), &["init", "b"]);
    let a = fs::read_to_string(t.path().join("AGENTS.md")).unwrap();
    assert!(a.starts_with("existing instruction\n"));
    assert_eq!(a.matches("gy ledger:").count(), 1);
    let nested = t.path().join("src/deep");
    fs::create_dir_all(&nested).unwrap();
    run(&nested, &["lint"]);
    reject(&nested, &["criterion", "add", "Criterion"], 2);
    run(
        &t.path().join("docs/ledger/a/criteria"),
        &["criterion", "add", "Criterion"],
    );
}
#[test]
fn workflow_cross_scope_edges_and_unknown_attributes() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    run(p, &["init", "b"]);
    run(
        p,
        &[
            "need",
            "add",
            "Delivery safety",
            "--targets",
            "AC-1",
            "--scope",
            "b",
        ],
    );
    run(
        p,
        &[
            "node",
            "set",
            "N-1",
            "--set",
            "custom={\"region\":\"east\",\"weight\":12}",
        ],
    );
    add_q(p);
    run(p, &["node", "set", "AC-1", "--set", "waiting-on=[\"Q-1\"]"]);
    let out = run(
        p,
        &[
            "decide",
            "Order deliveries by publication time",
            "--closes",
            "Q-1",
            "--scope-note",
            "Priority queue only",
            "--scope",
            "b",
        ],
    );
    assert!(
        out["open_questions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("AC-1"))
    );
    let q = run(p, &["show", "Q-1"]);
    assert_eq!(q["node"]["attrs"]["status"], "closed");
    assert!(q["display"].as_str().unwrap().contains("Decided by D-1"));
    let lint = invoke(p, &["lint"]);
    assert_eq!(lint.status.code(), Some(1));
    run(p, &["node", "set", "AC-1", "--set", "waiting-on=[]"]);
    run(p, &["lint"]);
    let hits = run(
        p,
        &[
            "find",
            "--where",
            "custom.region=east",
            "--where",
            "custom.weight>=10",
        ],
    );
    assert_eq!(hits["hits"].as_array().unwrap().len(), 1);
    assert_eq!(
        run(p, &["find", "--scope", "a", "--where", "type=need"])["hits"],
        json!([])
    );
    run(p, &["render"]);
    let n = run(p, &["show", "N-1"]);
    assert_eq!(n["node"]["attrs"]["custom"]["weight"], 12);
    let rendered = fs::read_to_string(p.join("docs/ledger/b/README.md")).unwrap();
    assert!(rendered.contains("region: east"));
}
#[test]
fn input_guards_explain_reasons_and_do_not_allocate() {
    let t = repo();
    let p = t.path();
    for (args, word) in [
        (vec!["need", "add", "need", "--scope", "a"], "overview"),
        (vec!["question", "add", "q", "--scope", "a"], "agreement"),
        (
            vec![
                "question",
                "add",
                "q",
                "--decider",
                "master",
                "--scope",
                "a",
            ],
            "facts",
        ),
        (vec!["decide", "d", "--scope", "a"], "too broadly"),
    ] {
        assert!(reject(p, &args, 2).contains(word));
    }
    add_d(p);
    assert_eq!(
        run(p, &["find", "--where", "type=decision"])["hits"][0]["id"],
        "D-1"
    );
    add_q(p);
    assert!(reject(p, &["question", "close", "Q-1", "--by", "decision"], 2).contains("gy decide"));
    reject(p, &["question", "close", "Q-1", "--by", "fact"], 2);
    reject(
        p,
        &[
            "question",
            "add",
            "How should deliveries be ordered?",
            "--decider",
            "master",
            "--options",
            "x",
            "--options",
            "y",
            "--scope",
            "a",
        ],
        2,
    );
    run(
        p,
        &[
            "question",
            "add",
            "How should deliveries be ordered?",
            "--decider",
            "master",
            "--options",
            "x",
            "--options",
            "y",
            "--force",
            "--scope",
            "a",
        ],
    );
    reject(
        p,
        &["need", "add", "need", "--targets", "AC-999", "--scope", "a"],
        2,
    );
    assert_eq!(run(p, &["find", "--where", "type=need"])["hits"], json!([]));
}
#[test]
fn q_capture_cannot_be_silenced_but_can_be_completed() {
    let t = repo();
    let p = t.path();
    run(p, &["q", "note", "--scope", "a"]);
    let path = p.join("docs/ledger/gy.toml");
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace("[lint]", "[lint]\nL8 = false\nL9 = false");
    fs::write(path, text).unwrap();
    reject(p, &["lint"], 1);
    run(
        p,
        &[
            "node",
            "set",
            "Q-1",
            "--set",
            "decider=master",
            "--set",
            "options=[\"a\",\"b\"]",
        ],
    );
    run(p, &["lint"]);
}
#[test]
fn requirement_parent_and_transition_guards() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    run(
        p,
        &["need", "add", "need", "--targets", "AC-1", "--scope", "a"],
    );
    run(p, &["req", "add", "req", "--issue", "6006", "--scope", "a"]);
    reject(p, &["need", "file", "N-1", "--issue", "6006"], 2);
    run(p, &["node", "set", "#6006", "--set", "parent_issue=6000"]);
    run(p, &["need", "file", "N-1", "--issue", "6006"]);
    reject(
        p,
        &[
            "req",
            "advance",
            "6006",
            "--to",
            "awaiting-merge",
            "--evidence",
            "verification",
        ],
        2,
    );
    run(
        p,
        &[
            "node",
            "set",
            "#6006",
            "--set",
            "pr_url=https://github.com/org/repo/pull/1",
            "--set",
            "pr_base=main",
            "--set",
            "pr_files=3",
        ],
    );
    reject(
        p,
        &[
            "req",
            "advance",
            "6006",
            "--to",
            "awaiting-merge",
            "--evidence",
            "verification",
            "--reported-base",
            "main",
            "--reported-files",
            "2",
        ],
        2,
    );
    run(
        p,
        &[
            "req",
            "advance",
            "6006",
            "--to",
            "awaiting-merge",
            "--evidence",
            "verification",
            "--reported-base",
            "main",
            "--reported-files",
            "3",
        ],
    );
    assert!(reject(p, &["req", "advance", "6006", "--to", "complete"], 2).contains("evidence"));
    run(
        p,
        &[
            "node",
            "set",
            "#6006",
            "--set",
            "remaining_work=0",
            "--set",
            "deviations=none",
            "--set",
            "residual=none",
        ],
    );
    reject(
        p,
        &[
            "req",
            "advance",
            "6006",
            "--to",
            "complete",
            "--evidence",
            "verification",
            "--data-migration",
            "false",
            "--production-only",
            "true",
            "--cleanup-done",
            "true",
        ],
        2,
    );
    run(
        p,
        &[
            "req",
            "advance",
            "6006",
            "--to",
            "awaiting-production",
            "--evidence",
            "verification",
            "--data-migration",
            "false",
            "--production-only",
            "true",
        ],
    );
    run(
        p,
        &[
            "req",
            "advance",
            "6006",
            "--to",
            "complete (phase 2)",
            "--evidence",
            "verification",
            "--data-migration",
            "false",
            "--production-only",
            "true",
            "--production-done",
            "true",
            "--cleanup-done",
            "true",
        ],
    );
    assert_eq!(
        run(p, &["show", "#6006"])["node"]["attrs"]["transitions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(run(p, &["next"])["nodes"], json!([]));
}
#[test]
fn all_thirteen_lint_rules_and_configured_severity() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    add_q(p);
    add_d(p);
    run(
        p,
        &["need", "add", "need", "--targets", "AC-1", "--scope", "a"],
    );
    run(p, &["req", "add", "req", "--issue", "6006", "--scope", "a"]);
    run(
        p,
        &["req", "add", "done", "--issue", "6007", "--scope", "a"],
    );
    run(
        p,
        &[
            "decide",
            "New constraint",
            "--scope-note",
            "Limited scope",
            "--scope",
            "a",
        ],
    );
    run(p, &["link", "D-2", "supersedes", "D-1"]);
    run(p, &["link", "#6006", "relies-on", "D-1"]);
    edit_node(p, "a/questions/Q-1.md", |n| {
        n.put("status", "closed");
        n.put("decider", "");
        n.put("options", json!(["one"]));
        n.put("bundle", "b");
        n.put("belongs-to", json!(["#6006", "#6007"]));
    });
    edit_node(p, "a/criteria/AC-1.md", |n| {
        n.put("waiting-on", json!(["Q-1"]));
        n.put("bearer_count", 9);
    });
    edit_node(p, "a/needs/N-1.md", |n| {
        n.put("targets", json!([]));
        n.put("depends-on", json!(["N-999"]));
    });
    edit_node(p, "a/decisions/D-1.md", |n| {
        n.put("decision_scope", "");
    });
    edit_node(p, "a/requirements/6006.md", |n| {
        n.put("status", "unknown");
    });
    edit_node(p, "a/requirements/6007.md", |n| {
        n.put("status", "complete");
        n.put("remaining_work", 1);
    });
    let o = invoke(p, &["lint"]);
    assert_eq!(o.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    for i in 1..=13 {
        assert!(
            v["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["rule"] == format!("L{i}")),
            "missing L{i}: {v}"
        );
    }
    let cfg = p.join("docs/ledger/gy.toml");
    let original = fs::read_to_string(&cfg).unwrap();
    let mut settings = String::new();
    for i in 1..=13 {
        settings.push_str(&format!(
            "L{i} = {{ enabled = true, severity = \"warn\" }}\n"
        ));
    }
    settings.push_str("edges = false\n");
    fs::write(
        &cfg,
        original.replace("[lint]", &format!("[lint]\n{settings}")),
    )
    .unwrap();
    let v = run(p, &["lint"]);
    assert!(
        v["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["severity"] == "warn")
    );
}
#[test]
fn marks_preserve_body_and_render_splits() {
    let t = repo();
    let p = t.path();
    add_d(p);
    run(p, &["node", "set", "D-1", "--set", "extra=kept"]);
    edit_node(p, "a/decisions/D-1.md", |n| {
        n.body = "\n## Context\nOrdering may be implementation-dependent.\n".into()
    });
    run(
        p,
        &[
            "decide",
            "Fixed order",
            "--scope-note",
            "Limited scope",
            "--scope",
            "a",
        ],
    );
    run(
        p,
        &[
            "link",
            "D-2",
            "narrows",
            "D-1",
            "--mark",
            "Ordering may be implementation-dependent",
        ],
    );
    let show = run(p, &["show", "D-1"]);
    assert!(
        show["display"]
            .as_str()
            .unwrap()
            .contains("⟦Applicability narrowed: D-2⟧")
    );
    assert!(!show["node"]["body"].as_str().unwrap().contains('⟦'));
    let config = p.join("docs/ledger/gy.toml");
    let text = fs::read_to_string(&config)
        .unwrap()
        .replace("split_threshold = 100", "split_threshold = 1");
    fs::write(config, text).unwrap();
    let rendered = run(p, &["render"]);
    assert_eq!(rendered["files"].as_array().unwrap().len(), 3);
    run(p, &["lint"]);
    run(p, &["render", "--format", "dot"]);
    assert!(
        run(p, &["show", "D-1", "--graph"])["dot"]
            .as_str()
            .unwrap()
            .contains("D-2")
    );
}
#[test]
fn next_waits_for_dependencies_and_questions() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    add_q(p);
    run(
        p,
        &["need", "add", "first", "--targets", "AC-1", "--scope", "a"],
    );
    run(
        p,
        &["need", "add", "second", "--targets", "AC-1", "--scope", "a"],
    );
    run(p, &["link", "N-2", "depends-on", "N-1"]);
    reject(p, &["link", "N-1", "depends-on", "N-2"], 2);
    assert_eq!(run(p, &["next"])["nodes"].as_array().unwrap().len(), 1);
    run(p, &["node", "set", "N-1", "--set", "waiting-on=[\"Q-1\"]"]);
    assert_eq!(run(p, &["next"])["nodes"], json!([]));
    run(
        p,
        &[
            "question",
            "close",
            "Q-1",
            "--by",
            "fact",
            "--note",
            "Determined by facts",
        ],
    );
    assert_eq!(run(p, &["next"])["nodes"].as_array().unwrap().len(), 1);
    run(p, &["node", "set", "N-1", "--set", "status=complete"]);
    assert_eq!(run(p, &["next"])["nodes"][0]["attrs"]["id"], "N-2");
}
#[test]
fn import_preserves_ids_and_is_atomic_on_conflict() {
    let t = repo();
    let p = t.path();
    let dir = p.join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(
        dir.join("0147-order.md"),
        "# Publication order\n\n## Context\nevidence\n",
    )
    .unwrap();
    fs::write(
        dir.join("D-148.md"),
        "---\nid: D-148\ntitle: Continuation\ncustom: preserved\n---\n\n## Context\nContext\n",
    )
    .unwrap();
    assert_eq!(
        run(p, &["import", "legacy", "--scope", "a"])["imported"],
        json!(["D-147", "D-148"])
    );
    assert_eq!(
        run(p, &["show", "D-148"])["node"]["attrs"]["custom"],
        "preserved"
    );
    assert_eq!(
        run(
            p,
            &[
                "decide",
                "Next",
                "--scope-note",
                "Limited scope",
                "--scope",
                "a"
            ]
        )["node"]["attrs"]["id"],
        "D-149"
    );
    fs::write(dir.join("0001-new.md"), "# new\n").unwrap();
    reject(p, &["import", "legacy", "--scope", "a"], 2);
    reject(p, &["show", "D-1"], 2);
}
#[test]
fn mcp_handshake_list_call_and_error_recovery() {
    let t = repo();
    let mut child = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(t.path())
        .args(["mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let input = child.stdin.as_mut().unwrap();
    for v in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"gy_criterion","arguments":{"args":["add","condition","--scope","a"]}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"gy_show","arguments":{"args":["D-999"]}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"gy_find","arguments":{"args":[]}}}),
    ] {
        writeln!(input, "{v}").unwrap();
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let lines: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 5);
    assert_eq!(lines[0]["result"]["protocolVersion"], "2025-11-25");
    assert!(lines[1]["result"]["tools"].as_array().unwrap().len() >= 15);
    assert_eq!(lines[2]["result"]["isError"], false);
    assert_eq!(lines[3]["result"]["isError"], true);
    assert_eq!(
        lines[4]["result"]["structuredContent"]["result"]["hits"][0]["id"],
        "AC-1"
    );
}
#[test]
fn skills_completions_help_and_corrupt_exit_code() {
    let t = repo();
    let p = t.path();
    assert_eq!(
        run(p, &["skills", "install", "skills"])["installed"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert!(
        run(p, &["completions", "zsh"])["script"]
            .as_str()
            .unwrap()
            .contains("_gy")
    );
    assert!(
        run(p, &["question", "close", "--help"])["message"]
            .as_str()
            .unwrap()
            .contains("--decision")
    );
    fs::write(p.join("docs/ledger/a/questions/Q-1.md"), "broken").unwrap();
    reject(p, &["lint"], 3);
}
#[test]
fn concurrent_allocation_is_global_and_unique() {
    let t = repo();
    let p = t.path();
    run(p, &["init", "b"]);
    let mut children = vec![];
    for i in 0..12 {
        children.push(
            Command::new(env!("CARGO_BIN_EXE_gy"))
                .current_dir(p)
                .args([
                    "--json",
                    "criterion",
                    "add",
                    &format!("criterion {i}"),
                    "--scope",
                    if i % 2 == 0 { "a" } else { "b" },
                ])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }
    let mut ids = std::collections::BTreeSet::new();
    for c in children {
        let o = c.wait_with_output().unwrap();
        assert!(o.status.success());
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        ids.insert(v["node"]["attrs"]["id"].as_str().unwrap().to_string());
    }
    assert_eq!(ids.len(), 12);
    run(p, &["lint"]);
}
#[test]
fn redo_journal_recovers_before_read() {
    let t = repo();
    let p = t.path();
    let n = gy_core::Node::new("AC-7", "criterion", "Recovered", "a");
    let j = json!({"files":{"a/criteria/AC-7.md":n.markdown().unwrap(),".gy-ids.json":"{\"criterion\":7}"}});
    fs::write(p.join("docs/ledger/.gy-transaction.json"), j.to_string()).unwrap();
    assert_eq!(
        run(p, &["show", "AC-7"])["node"]["attrs"]["title"],
        "Recovered"
    );
    assert!(!p.join("docs/ledger/.gy-transaction.json").exists());
    assert_eq!(
        run(p, &["criterion", "add", "next", "--scope", "a"])["node"]["attrs"]["id"],
        "AC-8"
    );
}
#[test]
fn handover_reports_facts_and_stats_use_git_additions() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    add_q(p);
    run(p, &["req", "add", "req", "--issue", "1", "--scope", "a"]);
    assert_eq!(invoke(p, &["handover"]).status.code(), Some(1));
    run(
        p,
        &[
            "node",
            "set",
            "#1",
            "--set",
            "next_evidence=review",
            "--set",
            "responsible=master",
        ],
    );
    let h = run(p, &["handover"]);
    assert_eq!(h["with_next_evidence_and_responsible"], 1);
    assert!(!h.to_string().contains("ready for handover"));
    for args in [
        vec!["init"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            "initial ledger",
        ],
    ] {
        assert!(
            Command::new("git")
                .current_dir(p)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    run(
        p,
        &[
            "criterion",
            "satisfy",
            "AC-1",
            "--evidence",
            "verification results",
        ],
    );
    let s = run(p, &["stats"]);
    assert_eq!(s["criteria"]["satisfied"], 1);
    assert_eq!(s["question_arrival"]["current_new_questions"], 1);
    reject(p, &["node", "set", "AC-1", "--set", "satisfied=false"], 2);
}

#[test]
fn compression_preview_requires_completed_work_and_each_constraint_decision() {
    let t = repo();
    let p = t.path();
    run(p, &["req", "add", "req", "--issue", "7", "--scope", "a"]);
    reject(p, &["req", "compress", "7"], 2);
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "remaining_work=0",
            "--set",
            "constraints_reviewed=true",
            "--set",
            "constraints=[{\"text\":\"Duplicate prevention is required\"}]",
        ],
    );
    edit_node(p, "a/requirements/7.md", |n| n.put("status", "complete"));
    assert!(reject(p, &["req", "compress", "7"], 2).contains("gy decide"));
    add_d(p);
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "constraints=[{\"text\":\"Duplicate prevention is required\",\"decision\":\"D-1\"}]",
        ],
    );
    reject(p, &["req", "compress", "7"], 2);
    run(p, &["link", "#7", "relies-on", "D-1"]);
    compression_fields(p, "#7");
    assert!(
        run(p, &["req", "compress", "7"])["archive"]
            .as_str()
            .unwrap()
            .contains("Duplicate prevention")
    );
}

#[test]
fn refiling_does_not_rewind_and_invalid_title_does_not_corrupt() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    run(
        p,
        &["need", "add", "need", "--targets", "AC-1", "--scope", "a"],
    );
    run(
        p,
        &[
            "req",
            "add",
            "req",
            "--issue",
            "7",
            "--parent-issue",
            "6000",
            "--scope",
            "a",
        ],
    );
    run(p, &["need", "file", "N-1", "--issue", "7"]);
    run(
        p,
        &[
            "req",
            "advance",
            "7",
            "--to",
            "awaiting-implementation",
            "--evidence",
            "approval record",
        ],
    );
    run(p, &["need", "file", "N-1", "--issue", "7"]);
    assert_eq!(
        run(p, &["show", "#7"])["node"]["attrs"]["status"],
        "awaiting-implementation"
    );
    reject(p, &["node", "set", "#7", "--set", "title="], 2);
    run(p, &["lint"]);
}

#[test]
fn handover_checks_dangling_references_even_with_l13_disabled() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    run(
        p,
        &["node", "set", "AC-1", "--set", "custom_reference=D-999"],
    );
    let path = p.join("docs/ledger/gy.toml");
    let config = fs::read_to_string(&path)
        .unwrap()
        .replace("[lint]", "[lint]\nL13 = false");
    fs::write(path, config).unwrap();
    run(p, &["lint"]);
    let out = invoke(p, &["handover"]);
    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["dangling"][0]["target"], "D-999");
}

fn compression_fields(p: &Path, id: &str) {
    run(
        p,
        &[
            "node",
            "set",
            id,
            "--set",
            "summary=Prevented duplicate order lines with a unique constraint.",
            "--set",
            "contracts_changed=[\"orders.order_lines (added UNIQUE constraint)\",\"POST /api/v1/orders (added 409 response)\"]",
            "--set",
            "artifacts={\"pr\":\"https://github.com/org/repo/pull/6003\",\"merge_commit\":\"a1b2c3d\",\"base_branch\":\"main\"}",
            "--set",
            "production={\"migration_total\":12431,\"migration_updated\":87,\"migration_remaining\":0,\"verified_env\":\"production\",\"verified_at\":\"2026-09-09\"}",
            "--set",
            "deviations=none",
            "--set",
            "residual=none",
        ],
    );
}

fn compressible() -> TempDir {
    let t = repo();
    let p = t.path();
    run(p, &["req", "add", "req", "--issue", "7", "--scope", "a"]);
    compression_fields(p, "#7");
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "constraints=[]",
            "--set",
            "constraints_reviewed=true",
        ],
    );
    run(
        p,
        &[
            "req",
            "advance",
            "7",
            "--to",
            "complete",
            "--evidence",
            "verification record",
            "--data-migration",
            "false",
            "--production-only",
            "false",
            "--cleanup-done",
            "true",
        ],
    );
    t
}

#[test]
fn compression_retains_six_records_edges_extensions_and_exact_archive() {
    let t = compressible();
    let p = t.path();
    add_d(p);
    run(p, &["link", "#7", "relies-on", "D-1"]);
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "custom={\"owner\":\"team-a\"}",
            "--set",
            "quality_gates={\"ci\":\"passed\"}",
            "--set",
            "design_proposal=Full text",
            "--set",
            "audit_records=Full audit text",
        ],
    );
    let path = p.join("docs/ledger/a/requirements/7.md");
    let raw = fs::read_to_string(&path).unwrap().replacen(
        "---\n",
        "---\n# preserve this comment in archive\n",
        1,
    ) + "\n## Audit records\nDetails removed after compression\n";
    fs::write(&path, &raw).unwrap();
    let preview = run(p, &["req", "compress", "7"]);
    assert_eq!(preview["archive"], raw);
    assert_eq!(fs::read_to_string(&path).unwrap(), raw);
    let output = run(
        p,
        &[
            "req",
            "compress",
            "7",
            "--evidence",
            "https://github.com/org/repo/issues/7#issuecomment-123",
        ],
    );
    assert_eq!(output["archive"], raw);
    let attrs = &output["node"]["attrs"];
    assert_eq!(attrs["id"], "#7");
    assert!(attrs["created"].is_string());
    assert_eq!(attrs["production"]["migration_total"], 12431);
    assert_eq!(attrs["production"]["migration_updated"], 87);
    assert_eq!(attrs["production"]["migration_remaining"], 0);
    assert_eq!(attrs["custom"]["owner"], "team-a");
    assert_eq!(attrs["relies-on"], json!(["D-1"]));
    for key in [
        "quality_gates",
        "design_proposal",
        "audit_records",
        "constraints",
        "constraints_reviewed",
        "remaining_work",
        "transitions",
    ] {
        assert!(attrs.get(key).is_none(), "{key} remained");
    }
    let body = output["node"]["body"].as_str().unwrap();
    assert_eq!(
        body.lines().filter(|line| line.starts_with("## ")).count(),
        6
    );
    assert!(!body.contains("Details removed after compression"));
    assert!(!body.contains("D-1"));
    assert!(!body.contains("issuecomment-123"));
    run(p, &["lint"]);
    run(p, &["handover"]);
    run(p, &["render"]);
    assert_eq!(
        run(
            p,
            &["find", "--where", "contracts_changed~orders.order_lines"]
        )["hits"][0]["id"],
        "#7"
    );
    assert!(
        run(p, &["show", "D-1"])["node"]["attrs"]["relied-on-by"]
            .as_array()
            .unwrap()
            .contains(&json!("#7"))
    );
    let compressed = fs::read_to_string(&path).unwrap();
    reject(
        p,
        &[
            "req",
            "compress",
            "7",
            "--evidence",
            "https://github.com/org/repo/issues/7#issuecomment-999",
        ],
        2,
    );
    assert_eq!(fs::read_to_string(path).unwrap(), compressed);
}

#[test]
fn compression_refuses_missing_empty_records_bad_archive_and_unknown_owners() {
    let t = compressible();
    let p = t.path();
    let path = p.join("docs/ledger/a/requirements/7.md");
    let original = fs::read_to_string(&path).unwrap();
    for field in [
        "summary",
        "contracts_changed",
        "artifacts",
        "production",
        "deviations",
        "residual",
    ] {
        for value in [None, Some(Value::Null), Some(json!("")), Some(json!([]))] {
            edit_node(p, "a/requirements/7.md", |n| {
                if let Some(value) = value {
                    n.attrs.insert(field.into(), value);
                } else {
                    n.attrs.remove(field);
                }
            });
            let before = fs::read_to_string(&path).unwrap();
            reject(
                p,
                &[
                    "req",
                    "compress",
                    "7",
                    "--evidence",
                    "https://github.com/org/repo/issues/7#issuecomment-1",
                ],
                2,
            );
            assert_eq!(fs::read_to_string(&path).unwrap(), before);
            if ["deviations", "residual"].contains(&field) {
                reject(p, &["lint"], 1);
            }
            fs::write(&path, &original).unwrap();
        }
    }
    for url in [
        "",
        "https:///issues/7#issuecomment-1",
        "https://github.com/org/repo/issues/8#issuecomment-1",
        "https://github.com/org/repo/pull/7",
        "https://github.com/org/repo/issues/7",
    ] {
        reject(p, &["req", "compress", "7", "--evidence", url], 2);
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }
    for residual in [
        "Still unaddressed",
        "[]",
        "[\"#7\"]",
        "[\"N-999\"]",
        "[\"D-1\"]",
    ] {
        run(
            p,
            &[
                "node",
                "set",
                "#7",
                "--set",
                &format!("residual={residual}"),
            ],
        );
        reject(p, &["lint"], 1);
        reject(p, &["req", "compress", "7"], 2);
        fs::write(&path, &original).unwrap();
    }
}

#[test]
fn delegated_residual_is_not_local_remaining_work_and_none_is_explicit() {
    let t = compressible();
    let p = t.path();
    add_ac(p);
    add_q(p);
    run(
        p,
        &[
            "need",
            "add",
            "Follow-up work",
            "--targets",
            "AC-1",
            "--scope",
            "a",
        ],
    );
    run(
        p,
        &[
            "req",
            "add",
            "Follow-up requirement",
            "--issue",
            "8",
            "--scope",
            "a",
        ],
    );
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "residual=[{\"id\":\"N-1\",\"note\":\"Transferred optimization work\"},\"Q-1\",\"#8\"]",
            "--set",
            "production=none",
        ],
    );
    run(p, &["lint"]);
    run(p, &["node", "set", "#7", "--set", "remaining_work=1"]);
    reject(p, &["req", "compress", "7"], 2);
    reject(p, &["lint"], 1);
    run(p, &["node", "set", "#7", "--set", "remaining_work=0"]);
    run(
        p,
        &[
            "req",
            "compress",
            "7",
            "--evidence",
            "https://github.com/org/repo/issues/7#issuecomment-9",
        ],
    );
    run(p, &["lint"]);
    edit_node(p, "a/requirements/7.md", |n| {
        n.attrs.remove("deviations");
    });
    reject(p, &["lint"], 1);
    run(p, &["node", "set", "#7", "--set", "deviations=none"]);
    run(p, &["lint"]);
}

#[test]
fn mcp_compression_uses_the_same_guard_and_writes_six_records() {
    let t = compressible();
    let p = t.path();
    let mut child = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(p)
        .args(["mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.as_mut().unwrap(),"{}",json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"gy_req","arguments":{"args":["compress","7","--evidence","https://github.com/org/repo/issues/7#issuecomment-88"]}}})).unwrap();
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["result"]["isError"], false);
    assert_eq!(
        v["result"]["structuredContent"]["result"]["node"]["attrs"]["compressed_from"],
        "https://github.com/org/repo/issues/7#issuecomment-88"
    );
    run(p, &["lint"]);
}

#[test]
fn requirement_targets_survive_compression_without_inflating_need_count() {
    let t = compressible();
    let p = t.path();
    add_ac(p);
    run(p, &["node", "set", "AC-1", "--set", "bearer_count=0"]);
    run(p, &["link", "#7", "targets", "AC-1"]);
    run(p, &["lint"]);
    run(
        p,
        &[
            "req",
            "compress",
            "7",
            "--evidence",
            "https://github.com/org/repo/issues/7#issuecomment-4",
        ],
    );
    assert_eq!(
        run(p, &["show", "#7"])["node"]["attrs"]["targets"],
        json!(["AC-1"])
    );
    assert_eq!(
        run(p, &["show", "AC-1"])["node"]["attrs"]["targeted-by"],
        json!(["#7"])
    );
    assert!(
        run(p, &["show", "#7", "--graph"])["dot"]
            .as_str()
            .unwrap()
            .contains("targets")
    );
    run(p, &["lint"]);
}

#[test]
fn compressed_show_and_edits_follow_frontmatter_and_guards_prevent_advance_without_none() {
    let t = compressible();
    let p = t.path();
    run(
        p,
        &[
            "req",
            "compress",
            "7",
            "--evidence",
            "https://github.com/org/repo/issues/7#issuecomment-4",
        ],
    );
    edit_node(p, "a/requirements/7.md", |n| {
        n.put("deviations", "Adjusted the response code after approval")
    });
    assert!(
        run(p, &["show", "#7"])["display"]
            .as_str()
            .unwrap()
            .contains("## 5. Deviations from the approved design and additional decisions\n\nAdjusted the response code after approval")
    );
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "summary=Deployed the adjusted response code to production.",
        ],
    );
    let raw = fs::read_to_string(p.join("docs/ledger/a/requirements/7.md")).unwrap();
    assert!(raw.contains(
        "## 1. One-sentence summary\n\nDeployed the adjusted response code to production."
    ));
    run(p, &["req", "add", "new", "--issue", "8", "--scope", "a"]);
    let args = [
        "req",
        "advance",
        "8",
        "--to",
        "complete",
        "--evidence",
        "verification",
        "--data-migration",
        "false",
        "--production-only",
        "false",
        "--cleanup-done",
        "true",
    ];
    assert!(reject(p, &args, 2).contains("residual"));
    run(
        p,
        &[
            "node",
            "set",
            "#8",
            "--set",
            "deviations=none",
            "--set",
            "residual=none",
        ],
    );
    run(p, &args);
}

#[test]
fn compression_preserves_archive_traceability_and_requires_actual_production_record() {
    let t = compressible();
    let p = t.path();
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "production=none",
            "--set",
            "data_migration=true",
        ],
    );
    assert!(reject(p, &["req", "compress", "7"], 2).contains("measurements"));
    run(
        p,
        &[
            "node",
            "set",
            "#7",
            "--set",
            "production={\"migration_total\":12431,\"migration_updated\":87,\"migration_remaining\":0}",
        ],
    );
    run(
        p,
        &[
            "req",
            "compress",
            "7",
            "--evidence",
            "https://github.com/org/repo/issues/7#issuecomment-4",
        ],
    );
    edit_node(p, "a/requirements/7.md", |n| {
        n.attrs.remove("compressed_from");
    });
    let result = invoke(p, &["lint"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stdout).contains("compressed_from"));
}

#[test]
fn english_body_hints_detect_waiting_references_and_bearer_counts() {
    let t = repo();
    let p = t.path();
    add_ac(p);
    add_q(p);
    edit_node(p, "a/criteria/AC-1.md", |n| {
        n.body = "\nWaiting on Q-1: unresolved\nbearers 3\n".into();
    });
    let closed = run(
        p,
        &[
            "question",
            "close",
            "Q-1",
            "--by",
            "fact",
            "--note",
            "Measurements resolved the question",
        ],
    );
    assert!(
        closed["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("AC-1"))
    );
    let output = invoke(p, &["lint"]);
    assert_eq!(output.status.code(), Some(1));
    let findings: Value = serde_json::from_slice(&output.stdout).unwrap();
    for rule in ["L1", "L2"] {
        assert!(
            findings["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["rule"] == rule)
        );
    }
}

#[test]
fn import_maps_scope_sections_and_reports_missing_values() {
    let t = repo();
    let p = t.path();
    let config = p.join("docs/ledger/gy.toml");
    fs::write(
        &config,
        fs::read_to_string(&config)
            .unwrap()
            .replace("[import]", "[import]\nscope_note_section = 'Applicability'")
            .replace(
                "scope_note_placeholders = []",
                "scope_note_placeholders = ['Not recorded']",
            ),
    )
    .unwrap();
    let dir = p.join("legacy");
    fs::create_dir(&dir).unwrap();
    let body = "# Decision\n\n```md\n## Applicability\nExample only\n```\n## Applicability ##\nProduction only\n### Exceptions\nExclude replay\n## Consequences\nNot part of scope\n";
    fs::write(dir.join("0001.md"), body).unwrap();
    fs::write(
        dir.join("0002.md"),
        "# Missing\n## Applicability\n\n## Consequences\n",
    )
    .unwrap();
    fs::write(
        dir.join("0003.md"),
        "# Placeholder\n## Applicability\nNot recorded\n",
    )
    .unwrap();
    fs::write(dir.join("0004.md"), "---\nid: D-4\ndecision_scope: Canonical value\ncustom: retained\n---\n## Applicability\nBody value\n").unwrap();
    let result = run(p, &["import", "legacy", "--scope", "a"]);
    assert_eq!(
        result["import_summary"]["missing_decision_scope"],
        json!(["D-2", "D-3"])
    );
    assert_eq!(result["import_summary"]["missing_decision_scope_count"], 2);
    let node = run(p, &["show", "D-1"]);
    assert_eq!(node["node"]["body"], body);
    assert_eq!(
        node["node"]["attrs"]["decision_scope"],
        "Production only\n### Exceptions\nExclude replay"
    );
    assert_eq!(
        run(p, &["show", "D-4"])["node"]["attrs"]["decision_scope"],
        "Canonical value"
    );
    let lint = invoke(p, &["lint"]);
    let lint: Value = serde_json::from_slice(&lint.stdout).unwrap();
    let missing: Vec<_> = lint["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["rule"] == "L7")
        .map(|d| d["id"].clone())
        .collect();
    assert_eq!(missing, vec![json!("D-2"), json!("D-3")]);

    let other = repo();
    let result = run(
        other.path(),
        &["import", dir.to_str().unwrap(), "--scope", "a"],
    );
    assert_eq!(result["import_summary"]["missing_decision_scope_count"], 3);
}

#[test]
fn import_rejects_ambiguous_sections_without_writing_nodes() {
    let t = repo();
    let p = t.path();
    let config = p.join("docs/ledger/gy.toml");
    fs::write(
        &config,
        fs::read_to_string(&config)
            .unwrap()
            .replace("[import]", "[import]\nscope_note_section = 'Scope'"),
    )
    .unwrap();
    let dir = p.join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("0001.md"), "# One\n## Scope\nValid\n").unwrap();
    fs::write(dir.join("0002.md"), "# Two\n## Scope\nOne\n## Scope\nTwo\n").unwrap();
    assert!(reject(p, &["import", "legacy", "--scope", "a"], 2).contains("Multiple sections"));
    assert_eq!(
        run(p, &["find", "--where", "type=decision"])["hits"],
        json!([])
    );
}

#[test]
fn question_provenance_can_have_multiple_requirements_but_ownership_is_single() {
    let t = repo();
    let p = t.path();
    add_q(p);
    for issue in ["1", "2"] {
        run(
            p,
            &[
                "req",
                "add",
                "Requirement",
                "--issue",
                issue,
                "--scope",
                "a",
            ],
        );
        run(p, &["link", &format!("#{issue}"), "raised", "Q-1"]);
    }
    run(p, &["node", "set", "Q-1", "--set", "belongs-to=[\"#1\"]"]);
    let lint = run(p, &["lint"]);
    assert!(
        !lint["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["rule"] == "L4")
    );
    run(
        p,
        &["node", "set", "Q-1", "--set", "belongs-to=[\"#1\",\"#2\"]"],
    );
    let output = invoke(p, &["lint"]);
    assert_eq!(output.status.code(), Some(1));
    let lint: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        lint["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["rule"] == "L4")
    );
}

#[test]
fn imported_missing_marks_remain_l6_and_are_distinct_from_new_links() {
    let t = repo();
    let p = t.path();
    let dir = p.join("legacy");
    fs::create_dir(&dir).unwrap();
    fs::write(
        dir.join("0001.md"),
        "---\nid: D-1\ndecision_scope: Scope\nsuperseded-by: [D-2]\n---\nOld passage\n",
    )
    .unwrap();
    fs::write(dir.join("0002.md"), "---\nid: D-2\ndecision_scope: Scope\nsupersedes: [{id: D-1, custom: retained}]\n---\nNew body\n").unwrap();
    let result = run(p, &["import", "legacy", "--scope", "a"]);
    assert_eq!(result["import_summary"]["missing_mark_count"], 1);
    let show = run(p, &["show", "D-2"]);
    assert_eq!(show["node"]["attrs"]["supersedes"][0]["custom"], "retained");
    assert_eq!(show["node"]["attrs"]["supersedes"][0]["imported"], true);
    let lint = run(p, &["lint"]);
    let finding = lint["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["rule"] == "L6")
        .unwrap();
    assert_eq!(finding["severity"], "warn");
    assert!(
        finding["message"]
            .as_str()
            .unwrap()
            .contains("Imported relationship")
    );
    run(
        p,
        &["link", "D-2", "supersedes", "D-1", "--mark", "Old passage"],
    );
    assert!(
        !run(p, &["lint"])["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["rule"] == "L6")
    );
    run(p, &["link", "D-2", "supersedes", "D-1"]);
    let lint = run(p, &["lint"]);
    let finding = lint["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["rule"] == "L6")
        .unwrap();
    assert!(
        !finding["message"]
            .as_str()
            .unwrap()
            .contains("Imported relationship")
    );
}
