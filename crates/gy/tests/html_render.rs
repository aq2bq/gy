use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
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
fn payload(html_path: &Path) -> Value {
    let html = fs::read_to_string(html_path).unwrap();
    let start = html.find("window.GY_DATA = ").unwrap() + "window.GY_DATA = ".len();
    let end = html[start..].find(";\n</script>").unwrap() + start;
    serde_json::from_str(&html[start..end]).unwrap()
}

#[test]
fn render_html_generates_single_file_with_json_and_common_flags() {
    let t = repo();
    let p = t.path();
    run(p, &["criterion", "add", "AC", "--scope", "a"]);
    run(
        p,
        &["need", "add", "Need", "--targets", "AC-1", "--scope", "a"],
    );
    let out = run(p, &["render", "--format", "html"]);
    assert_eq!(out["files"].as_array().unwrap().len(), 1);
    let path = p.join("docs/ledger/gy.html");
    assert!(path.is_file());
    let html = fs::read_to_string(&path).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert_eq!(html.matches("<script id=\"gy-app\">").count(), 1);

    // --scope filters the payload to that scope
    let scoped = run(p, &["render", "--format", "html", "--scope", "a"]);
    assert_eq!(scoped["files"].as_array().unwrap().len(), 1);

    // --quiet suppresses human output but HTML is still written
    let o = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(p)
        .args(["render", "--format", "html", "--quiet"])
        .output()
        .unwrap();
    assert!(o.status.success());
    assert!(o.stdout.is_empty());

    // Nonexistent scope is an input error
    reject(p, &["render", "--format", "html", "--scope", "zzz"], 2);
}

#[test]
fn render_html_applies_output_guards_with_exit_2() {
    let t = repo();
    let p = t.path();
    run(p, &["criterion", "add", "AC", "--scope", "a"]);

    let config = p.join("docs/ledger/gy.toml");
    let into_nodes = "[render]\nsplit_threshold = 100\noutput = \"{scope}/README.md\"\nhtml_output = \"a/needs/evil.html\"\n";
    fs::write(&config, into_nodes).unwrap();
    let err = reject(p, &["render", "--format", "html"], 2);
    assert!(err.contains("source node directories"), "{err}");

    let overlap = "[render]\nsplit_threshold = 100\noutput = \"{scope}/README.md\"\nhtml_output = \"gy.toml\"\n";
    fs::write(&config, overlap).unwrap();
    let err = reject(p, &["render", "--format", "html"], 2);
    assert!(
        err.contains("overlaps a source or management file"),
        "{err}"
    );
}

#[test]
fn render_html_escapes_hostile_body_without_breaking_the_page() {
    let t = repo();
    let p = t.path();
    run(p, &["criterion", "add", "AC", "--scope", "a"]);
    run(
        p,
        &["need", "add", "Need", "--targets", "AC-1", "--scope", "a"],
    );
    run(p, &["decide", "D", "--scope-note", "notes", "--scope", "a"]);

    let hostile = "</script><p>hi</p><!---->\n\u{2028}\u{2029}x";
    // Write the hostile text directly into the node body (frontmatter unchanged)
    let node = p.join("docs/ledger/a/decisions/D-1.md");
    let raw = fs::read_to_string(&node).unwrap();
    let (yaml, _body) = raw.split_once("\n---\n").unwrap();
    fs::write(&node, format!("{yaml}\n---\n{hostile}")).unwrap();

    let out = run(p, &["render", "--format", "html"]);
    let path = p.join("docs/ledger/gy.html");
    // The payload region must be a decodable JSON object and terminate only at
    // the enclosing script block.
    let got = payload(&path);
    let d = got["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "D-1")
        .unwrap();
    assert_eq!(d["body"], json!(hostile));
    let _ = out;
}

#[test]
fn render_html_embeds_unknown_attributes_and_front_screen_numbers() {
    let t = repo();
    let p = t.path();
    run(p, &["criterion", "add", "AC", "--scope", "a"]);
    run(
        p,
        &["need", "add", "Need", "--targets", "AC-1", "--scope", "a"],
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
    let got = run(p, &["render", "--format", "html"]);
    let path = p.join("docs/ledger/gy.html");
    let html = fs::read_to_string(&path).unwrap();
    assert!(html.contains("east"));
    let d = payload(&path);
    let n = d["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "N-1")
        .unwrap();
    assert_eq!(n["attrs"]["custom"]["region"], "east");
    assert!(d["generated_at"].is_string());
    assert_eq!(d["node_count"], d["nodes"].as_array().unwrap().len() as u64);
    let _ = got;
}

#[test]
fn render_html_uses_html_output_and_splits_on_scope_placeholder() {
    let t = repo();
    let p = t.path();
    run(p, &["criterion", "add", "AC", "--scope", "a"]);
    run(p, &["init", "b"]);
    run(p, &["init", "c"]);
    run(p, &["criterion", "add", "AC-B", "--scope", "b"]);
    let config = p.join("docs/ledger/gy.toml");
    fs::write(
        &config,
        "[scopes.a]\n[scopes.b]\n[scopes.c]\n\n[render]\nsplit_threshold = 100\noutput = \"{scope}/README.md\"\nhtml_output = \"views/{scope}.html\"\n",
    )
    .unwrap();
    let out = run(p, &["render", "--format", "html"]);
    let files: Vec<_> = out["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap().to_string())
        .collect();
    assert_eq!(files.len(), 3);
    assert!(files.iter().any(|f| f.ends_with("views/a.html")));
    assert!(p.join("docs/ledger/views/a.html").is_file());
    assert!(p.join("docs/ledger/views/b.html").is_file());
    assert!(p.join("docs/ledger/views/c.html").is_file());
    // split_threshold never applies to html
    let a = payload(&p.join("docs/ledger/views/a.html"));
    assert_eq!(a["node_count"], 1);
}

#[test]
fn render_html_succeeds_with_lint_errors_and_exit_zero() {
    let t = repo();
    let p = t.path();
    // A requirement state outside the allowed set is an L10 error.
    let d = p.join("docs/ledger/a/requirements");
    let f = d.join("#6006.md");
    fs::write(
        &f,
        "---\nid: \"#6006\"\ntype: \"requirement\"\ntitle: \"Bad state\"\nscope: \"a\"\ncreated: \"2026-09-01\"\nstatus: \"bogus\"\n---\n",
    )
    .unwrap();
    let lint = invoke(p, &["lint"]);
    assert_eq!(lint.status.code(), Some(1));
    let out = run(p, &["render", "--format", "html"]);
    assert_eq!(out["files"].as_array().unwrap().len(), 1);
    let d = payload(&p.join("docs/ledger/gy.html"));
    assert!(
        d["lint"]
            .as_array()
            .unwrap()
            .iter()
            .any(|di| di["rule"] == "L10")
    );
}

#[test]
fn init_appends_default_html_output_to_ledger_gitignore() {
    let t = repo();
    let p = t.path();
    let ignore = p.join("docs/ledger/.gitignore");
    assert!(ignore.is_file());
    let text = fs::read_to_string(&ignore).unwrap();
    assert!(text.contains("gy.html"), "got: {text}");
    // Re-running init does not duplicate the entry
    run(p, &["init", "b"]);
    let text2 = fs::read_to_string(&ignore).unwrap();
    assert_eq!(text2.matches("gy.html").count(), 1);
}
