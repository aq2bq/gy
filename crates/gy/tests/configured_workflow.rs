use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const CONFIG: &str = r#"
[workflow.records.design]
kinds = ["requirement"]
version_field = "revision"
[workflow.records.design.fields]
revision = { type = "string" }
contracts = { type = "array", items = { type = "string" } }
files = { type = "array", items = { type = "string" } }
gates = { type = "array", items = { type = "object", fields = { contract = { type = "string" }, id = { type = "string" } } } }
plan = { type = "object", variant_field = "mode", fields = { mode = { type = "string" } }, variants = { normal = { url = { type = "url" } }, design-waived = { reason = { type = "string" }, stop_conditions = { type = "array", items = { type = "string" } } } } }

[workflow.records.approval]
kinds = ["requirement"]
version_field = "revision"
[workflow.records.approval.fields]
revision = { type = "string" }
target = { type = "string" }
approver = { type = "string" }
url = { type = "url" }

[workflow.records.report]
kinds = ["requirement"]
version_field = "revision"
[workflow.records.report.fields]
revision = { type = "string" }
target = { type = "string" }
files = { type = "array", items = { type = "string" } }
gates = { type = "array", items = { type = "object", fields = { contract = { type = "string" }, gate = { type = "string" }, command = { type = "string" }, population = { type = "integer" }, result = { type = "string", values = ["passed"] } } } }
deviations = { type = "object", variant_field = "status", fields = { status = { type = "string" } }, variants = { none = {}, reported = { details = { type = "string" } }, not-executed = { reason = { type = "string" } } } }

[workflow.records.research]
kinds = ["need", "question"]
required = true
version_field = "revision"
[workflow.records.research.fields]
revision = { type = "string" }
question = { type = "string" }
population = { type = "integer" }
verified = { type = "boolean" }
sources = { type = "array", items = { type = "string" } }
limits = { type = "array", allow_empty = true, items = { type = "string" } }

[workflow.guards.design]
states = ["awaiting-approval"]
records = ["design"]
[workflow.guards.approval]
states = ["awaiting-implementation", "awaiting-audit", "awaiting-pr"]
records = ["design", "approval"]
[[workflow.guards.approval.checks]]
kind = "equal"
left = "approval.target"
right = "design.revision"

[workflow.guards.audit]
states = ["awaiting-audit", "awaiting-pr"]
records = ["design", "report"]
[[workflow.guards.audit.checks]]
kind = "equal"
left = "report.target"
right = "design.revision"
[[workflow.guards.audit.checks]]
kind = "same-set"
left = "design.files"
right = "report.files"
[[workflow.guards.audit.checks]]
kind = "same-set"
left = "design.contracts"
right = "report.gates[].contract"
[[workflow.guards.audit.checks]]
kind = "same-set"
left = "design.gates"
right = "report.gates"
left_keys = ["contract", "id"]
right_keys = ["contract", "gate"]
"#;
fn invoke(p: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(p)
        .arg("--json")
        .args(args)
        .output()
        .unwrap()
}
fn run(p: &Path, args: &[&str]) -> Value {
    let o = invoke(p, args);
    assert!(
        o.status.success(),
        "{args:?}: {} {}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}
fn rejected(p: &Path, args: &[&str], text: &str) {
    let o = invoke(p, args);
    assert_eq!(
        o.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        String::from_utf8_lossy(&o.stderr).contains(text),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}
fn repo() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    run(t.path(), &["init", "test"]);
    fs::write(t.path().join("docs/ledger/gy.toml"), CONFIG).unwrap();
    run(
        t.path(),
        &["req", "add", "Feature", "--issue", "1", "--scope", "test"],
    );
    t
}
fn put(p: &Path, id: &str, name: &str, value: Value) {
    run(p, &["node", "set", id, "--set", &format!("{name}={value}")]);
}
fn design(revision: &str) -> Value {
    json!({"revision":revision,"contracts":["api","db"],"files":["src/api.rs","schema.sql"],
        "gates":[{"contract":"api","id":"test-api"},{"contract":"db","id":"test-db"}],
        "plan":{"mode":"normal","url":"https://example.test/issues/1#issuecomment-1"}})
}
fn approval(revision: &str, target: &str) -> Value {
    json!({"revision":revision,"target":target,"approver":"human-reviewer","url":"https://example.test/issues/1#issuecomment-2"})
}
fn report(target: &str) -> Value {
    json!({"revision":"r1","target":target,"files":["schema.sql","src/api.rs"],"deviations":{"status":"none"},
        "gates":[{"contract":"api","gate":"test-api","command":"test api","population":0,"result":"passed"},
                 {"contract":"db","gate":"test-db","command":"test db","population":2,"result":"passed"}]})
}
fn advance(p: &Path, state: &str) -> Value {
    run(
        p,
        &[
            "req",
            "advance",
            "1",
            "--to",
            state,
            "--evidence",
            "Reviewed reported inputs",
        ],
    )
}
fn reject_advance(p: &Path, state: &str, message: &str) {
    rejected(
        p,
        &[
            "req",
            "advance",
            "1",
            "--to",
            state,
            "--evidence",
            "Checked",
        ],
        message,
    );
}
fn lint(p: &Path) -> Value {
    let output = invoke(p, &["lint"]);
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn required_records_are_checked_on_every_entry_and_failures_are_atomic() {
    let t = repo();
    let p = t.path();
    let path = p.join("docs/ledger/test/requirements/1.md");
    let before = fs::read(&path).unwrap();
    reject_advance(p, "awaiting-implementation", "approval is required");
    assert_eq!(fs::read(&path).unwrap(), before);
    put(p, "#1", "design", design("d1"));
    advance(p, "awaiting-approval");
    put(p, "#1", "approval", approval("a1", "d1"));
    advance(p, "awaiting-implementation");
    put(p, "#1", "report", report("d1"));
    let node = advance(p, "awaiting-audit")["node"].clone();
    let snapshot = &node["attrs"]["transitions"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()["workflow"];
    assert_eq!(
        snapshot["records"]["approval"]["approver"],
        "human-reviewer"
    );
    assert_eq!(snapshot["records"]["design"]["revision"], "d1");
    assert!(snapshot["schemas"]["report"]["fields"]["gates"].is_object());
    assert_eq!(snapshot["checks"].as_array().unwrap().len(), 5);
    assert!(lint(p)["diagnostics"].as_array().unwrap().is_empty());
    // New state guards cannot be bypassed by disabling their lint output.
    let cfg = fs::read_to_string(p.join("docs/ledger/gy.toml")).unwrap();
    fs::write(
        p.join("docs/ledger/gy.toml"),
        cfg.replace("[lint]", "[lint]\nworkflow = 'off'"),
    )
    .unwrap();
    let mut broken = approval("a2", "old");
    broken["approver"] = json!("");
    put(p, "#1", "approval", broken);
    assert!(lint(p)["diagnostics"].as_array().unwrap().is_empty());
    reject_advance(p, "awaiting-pr", "approval.approver must not be empty");
}
#[test]
fn changed_design_requires_new_version_and_new_approval_and_history_is_preserved() {
    let t = repo();
    let p = t.path();
    put(p, "#1", "design", design("d1"));
    put(p, "#1", "approval", approval("a1", "d1"));
    let before = advance(p, "awaiting-implementation")["node"]["attrs"]["transitions"].clone();
    let mut changed = design("d1");
    changed["files"] = json!(["new.rs"]);
    put(p, "#1", "design", changed.clone());
    reject_advance(
        p,
        "awaiting-approval",
        "already recorded with different contents",
    );
    changed["revision"] = json!("d2");
    put(p, "#1", "design", changed);
    advance(p, "awaiting-approval (design revision)");
    reject_advance(p, "awaiting-implementation", "Equal check failed");
    put(p, "#1", "approval", approval("a2", "d2"));
    let current = advance(p, "awaiting-implementation")["node"]["attrs"]["transitions"].clone();
    assert_eq!(current[0], before[0]);
    assert_eq!(
        current[0]["workflow"]["records"]["design"]["files"],
        json!(["src/api.rs", "schema.sql"])
    );
    rejected(
        p,
        &["node", "set", "#1", "--set", "transitions=[]"],
        "dedicated command",
    );
}
#[test]
fn explicit_waiver_requires_reason_stops_and_approval() {
    let t = repo();
    let p = t.path();
    let mut d = design("d1");
    d["plan"] = json!({"mode":"design-waived","reason":"Emergency repair","stop_conditions":[]});
    put(p, "#1", "design", d.clone());
    reject_advance(
        p,
        "awaiting-implementation",
        "stop_conditions must not be empty",
    );
    d["plan"]["stop_conditions"] = json!(["Contract changes"]);
    put(p, "#1", "design", d);
    reject_advance(p, "awaiting-implementation", "approval is required");
    put(p, "#1", "approval", approval("a1", "d1"));
    advance(p, "awaiting-implementation");
}
#[test]
fn gate_contract_pairs_files_and_reported_failures_are_checked() {
    let t = repo();
    let p = t.path();
    put(p, "#1", "design", design("d1"));
    put(p, "#1", "approval", approval("a1", "d1"));
    for bad in ["swapped", "missing", "outside", "failed", "unknown-status"] {
        let mut r = report("d1");
        match bad {
            "swapped" => {
                r["gates"][0]["contract"] = json!("db");
                r["gates"][1]["contract"] = json!("api");
            }
            "missing" => {
                r["gates"].as_array_mut().unwrap().pop();
            }
            "outside" => {
                r["files"].as_array_mut().unwrap().push(json!("outside.rs"));
            }
            "failed" => {
                r["gates"][0]["result"] = json!("failed");
            }
            _ => {
                r["deviations"] = json!({"status":"not-executed"});
            }
        }
        put(p, "#1", "report", r);
        let o = invoke(
            p,
            &[
                "req",
                "advance",
                "1",
                "--to",
                "awaiting-audit",
                "--evidence",
                "Reported",
            ],
        );
        assert_eq!(o.status.code(), Some(2), "{bad}");
    }
    put(p, "#1", "report", report("d1"));
    advance(p, "awaiting-audit");
}
#[test]
fn research_submission_is_stateless_and_rejects_missing_and_mistyped_values() {
    let t = repo();
    let p = t.path();
    run(p, &["q", "Research question", "--scope", "test"]);
    assert!(
        lint(p)["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["rule"] == "workflow" && d["message"] == "research is required")
    );
    let mut record = json!({"revision":"r1","question":"How many?","population":0,"verified":false,"sources":["query.sql"],"limits":[]});
    record["population"] = json!("zero");
    put(p, "Q-1", "research", record.clone());
    rejected(
        p,
        &[
            "node",
            "submit",
            "Q-1",
            "--record",
            "research",
            "--evidence",
            "Run log",
        ],
        "Integer",
    );
    record["population"] = json!(0);
    put(p, "Q-1", "research", record.clone());
    let result = run(
        p,
        &[
            "node",
            "submit",
            "Q-1",
            "--record",
            "research",
            "--evidence",
            "Run log",
        ],
    );
    assert_eq!(result["node"]["attrs"]["status"], "open");
    assert_eq!(result["submission"]["records"]["research"], record);
    let output = invoke(p, &["handover"]);
    let handover: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        handover["workflow"]["records"]["research"]["required"],
        true
    );
    record["question"] = json!("Changed under same version");
    put(p, "Q-1", "research", record);
    rejected(
        p,
        &[
            "node",
            "submit",
            "Q-1",
            "--record",
            "research",
            "--evidence",
            "Run log",
        ],
        "new version",
    );
    rejected(
        p,
        &["node", "set", "Q-1", "--set", "record_history=[]"],
        "dedicated command",
    );
}
#[test]
fn invalid_configuration_fails_before_mutation() {
    let t = repo();
    let p = t.path();
    for invalid in [
        CONFIG.replace("type = \"integer\"", "type = \"integre\""),
        CONFIG.replace(
            "type = \"integer\"",
            "type = \"integer\", equals = 1, items = { type = \"string\" }",
        ),
        CONFIG.replace("report.target", "report.typo"),
        CONFIG.replace(
            "right_keys = [\"contract\", \"gate\"]",
            "right_keys = [\"contract\", \"typo\"]",
        ),
        CONFIG.replace(
            "states = [\"awaiting-approval\"]",
            "states = [\"not-a-state\"]",
        ),
        CONFIG.replace("fields]\nrevision", "fields]\nunknown"),
    ] {
        fs::write(p.join("docs/ledger/gy.toml"), invalid).unwrap();
        let o = invoke(
            p,
            &[
                "req",
                "advance",
                "1",
                "--to",
                "awaiting-approval",
                "--evidence",
                "Checked",
            ],
        );
        assert_eq!(
            o.status.code(),
            Some(3),
            "{}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

#[test]
fn subset_checks_reported_files_against_allowed_files() {
    let t = repo();
    let p = t.path();
    let config = CONFIG.replace(
        "kind = \"same-set\"\nleft = \"design.files\"\nright = \"report.files\"",
        "kind = \"subset\"\nleft = \"report.files\"\nright = \"design.files\"",
    );
    fs::write(p.join("docs/ledger/gy.toml"), config).unwrap();
    put(p, "#1", "design", design("d1"));
    put(p, "#1", "approval", approval("a1", "d1"));
    let mut r = report("d1");
    r["files"] = json!(["outside.rs"]);
    put(p, "#1", "report", r.clone());
    reject_advance(p, "awaiting-audit", "Subset check failed");
    r["files"] = json!(["src/api.rs", "src/api.rs"]);
    put(p, "#1", "report", r);
    advance(p, "awaiting-audit");
}

#[test]
fn history_reference_checks_read_evidence_and_records_but_not_schema_constants() {
    let snapshot = json!({
        "records": {"research": {"source": "D-12"}},
        "evidence": "Q-13",
        "schemas": {"research": {"fields": {"source": {"values": ["D-999"]}}}},
        "checks": []
    });
    assert_eq!(
        gy_core::recorded_attribute_ids("record_history", &json!([snapshot.clone()])),
        ["D-12".to_string(), "Q-13".to_string()]
            .into_iter()
            .collect()
    );
    assert_eq!(
        gy_core::recorded_attribute_ids(
            "transitions",
            &json!([{
                "evidence": "Q-13", "workflow": snapshot
            }])
        ),
        ["D-12".to_string(), "Q-13".to_string()]
            .into_iter()
            .collect()
    );
}
#[test]
fn shipped_workflow_example_is_valid_and_missing_legacy_records_are_visible() {
    let t = repo();
    let p = t.path();
    fs::write(
        p.join("docs/ledger/gy.toml"),
        include_str!("../examples/workflow.toml"),
    )
    .unwrap();
    // Edit legacy state directly, as would happen during migration.
    let file = p.join("docs/ledger/test/requirements/1.md");
    let source = fs::read_to_string(&file)
        .unwrap()
        .replace("status: defining", "status: awaiting-audit");
    fs::write(file, source).unwrap();
    let diagnostics = lint(p);
    assert!(
        diagnostics["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["rule"] == "workflow" && d["message"] == "quality_gates is required")
    );
}

#[test]
fn example_records_cover_normal_and_waived_workflows_and_archive_history() {
    let t = repo();
    let p = t.path();
    fs::write(
        p.join("docs/ledger/gy.toml"),
        include_str!("../examples/workflow.toml"),
    )
    .unwrap();
    let records: Value =
        serde_json::from_str(include_str!("../examples/workflow-records.json")).unwrap();
    put(p, "#1", "dispatch", records["dispatch"].clone());
    advance(p, "awaiting-design");
    put(
        p,
        "#1",
        "design_proposal",
        records["design_proposal"].clone(),
    );
    advance(p, "awaiting-approval");
    put(p, "#1", "approval", records["approval"].clone());
    advance(p, "awaiting-implementation");
    for name in ["quality_gates", "deviations", "implementation_report"] {
        put(p, "#1", name, records[name].clone());
    }
    advance(p, "awaiting-audit");
    reject_advance(p, "awaiting-pr", "audit_records is required");
    put(p, "#1", "audit_records", records["audit_records"].clone());
    advance(p, "awaiting-pr");
    // A declared out-of-scope change fails even if the caller's file list agrees.
    let mut changed = records["implementation_report"].clone();
    changed["revision"] = json!("implementation-v2");
    changed["out_of_scope_changed"] = json!(true);
    put(p, "#1", "implementation_report", changed);
    reject_advance(p, "awaiting-pr", "must equal false");
    put(
        p,
        "#1",
        "implementation_report",
        records["implementation_report"].clone(),
    );
    // Research submission on a need does not advance an associated requirement.
    run(p, &["criterion", "add", "Criterion", "--scope", "test"]);
    run(
        p,
        &[
            "need",
            "add",
            "Research needed",
            "--targets",
            "AC-1",
            "--scope",
            "test",
        ],
    );
    put(p, "N-1", "research", records["research"].clone());
    run(
        p,
        &[
            "node",
            "submit",
            "N-1",
            "--record",
            "research",
            "--evidence",
            "Research log",
        ],
    );
    assert_eq!(
        run(p, &["show", "N-1"])["node"]["attrs"]["status"],
        "unfiled"
    );

    // No normal design proposal is required for an explicitly approved waiver.
    run(
        p,
        &[
            "req",
            "add",
            "Emergency fix",
            "--issue",
            "2",
            "--scope",
            "test",
        ],
    );
    put(p, "#2", "dispatch", records["dispatch"].clone());
    let mut waived = records["design_proposal"].clone();
    waived["plan"] = json!({"mode":"design-waived","url":"https://example.test/issues/2#issuecomment-20",
        "reason":"Emergency repair","stop_conditions":["Stop on contract changes"]});
    put(p, "#2", "design_proposal", waived);
    let mut approved = records["approval"].clone();
    approved["mode"] = json!("design-waived");
    put(p, "#2", "approval", approved);
    run(
        p,
        &[
            "req",
            "advance",
            "2",
            "--to",
            "awaiting-implementation",
            "--evidence",
            "Waiver approved",
        ],
    );
    assert!(lint(p)["diagnostics"].as_array().unwrap().is_empty());

    // Existing completion and archive guards remain in force.
    run(
        p,
        &[
            "node",
            "submit",
            "#1",
            "--record",
            "design_proposal",
            "--evidence",
            "Review record",
        ],
    );
    for (key, value) in [
        ("summary", json!("Made retries idempotent.")),
        ("contracts_changed", json!("Retry idempotency")),
        (
            "artifacts",
            json!({"pr":"https://example.test/pull/3","merge_commit":"abcdef1","base_branch":"main"}),
        ),
        ("production", json!("none")),
        ("residual", json!("none")),
        ("constraints", json!([])),
        ("constraints_reviewed", json!(true)),
    ] {
        put(p, "#1", key, value);
    }
    run(
        p,
        &[
            "req",
            "advance",
            "1",
            "--to",
            "complete",
            "--evidence",
            "Completed",
            "--data-migration",
            "false",
            "--production-only",
            "false",
            "--cleanup-done",
            "true",
        ],
    );
    let compressed = run(
        p,
        &[
            "req",
            "compress",
            "1",
            "--evidence",
            "https://example.test/issues/1#issuecomment-99",
        ],
    );
    assert!(
        compressed["archive"]
            .as_str()
            .unwrap()
            .contains("record_history:")
    );
    assert!(compressed["node"]["attrs"].get("record_history").is_none());
    assert!(lint(p)["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn mcp_submission_and_history_corruption_use_the_same_checks() {
    use std::io::Write;
    use std::process::Stdio;
    let t = repo();
    let p = t.path();
    run(p, &["q", "Research", "--scope", "test"]);
    let mut child = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(p)
        .args(["mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"gy_node","arguments":{"args":["submit","Q-1","--record","research","--evidence","Log"]}}});
    writeln!(child.stdin.take().unwrap(), "{request}").unwrap();
    let output = child.wait_with_output().unwrap();
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["result"]["isError"], true);
    assert!(result.to_string().contains("research is required"));
    put(
        p,
        "Q-1",
        "research",
        json!({"revision":"r1","question":"Why?","population":0,"verified":false,"sources":["log"],"limits":[]}),
    );
    run(
        p,
        &[
            "node",
            "submit",
            "Q-1",
            "--record",
            "research",
            "--evidence",
            "Log",
        ],
    );
    let file = p.join("docs/ledger/test/questions/Q-1.md");
    let mut node = gy_core::Node::parse(&fs::read_to_string(&file).unwrap(), file.clone()).unwrap();
    node.put("record_history", json!({}));
    fs::write(file, node.markdown().unwrap()).unwrap();
    rejected(
        p,
        &[
            "node",
            "submit",
            "Q-1",
            "--record",
            "research",
            "--evidence",
            "Log",
        ],
        "record_history must be an array",
    );
    assert!(
        lint(p)["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["rule"] == "workflow"
                && d["message"].as_str().unwrap().contains("record_history"))
    );
}
