//! A write's output shape is a contract (n-8a28, ac-27e6): a creating write
//! prints `id: <ID>` first, a write that returns an id the caller knows keeps it
//! bare, and either is followed by `changed:` / `missing:` / `next:` /
//! `unresolved:` in that order. `--json` carries the same fields as top-level
//! keys.
mod common;

use common::fixture;
use serde_json::Value;
use std::process::Output;

const SHAPE: &str = "the output shape is a contract: if you changed it on purpose, write it under Changed in CHANGELOG.md";

fn ok(output: &Output) -> Vec<String> {
    assert!(
        output.status.success(),
        "{}: the write failed: {}",
        SHAPE,
        common::stderr(output)
    );
    common::stdout(output).lines().map(str::to_string).collect()
}

/// Check the five-line body every write shares and return the id line.
fn five(output: &Output) -> String {
    let lines = ok(output);
    assert!(
        lines.len() == 5,
        "{}: a write prints 5 lines, got {}: {:?}",
        SHAPE,
        lines.len(),
        lines
    );
    assert!(
        lines[1].starts_with("changed: "),
        "{}: line 2 must start with `changed: `, got {:?}",
        SHAPE,
        lines[1]
    );
    assert!(
        lines[2].starts_with("missing: "),
        "{}: line 3 must start with `missing: `, got {:?}",
        SHAPE,
        lines[2]
    );
    assert!(
        lines[3].starts_with("next: "),
        "{}: line 4 must start with `next: `, got {:?}",
        SHAPE,
        lines[3]
    );
    assert!(
        lines[4].starts_with("unresolved: "),
        "{}: line 5 must start with `unresolved: `, got {:?}",
        SHAPE,
        lines[4]
    );
    lines[0].clone()
}

fn first_id(line: &str) -> String {
    line.split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

/// A creating write: `id: <ID>`, or `id: <ID> (<ref>)` for a requirement.
fn creating(output: &Output) -> String {
    let line = five(output);
    let rest = line.strip_prefix("id: ").unwrap_or_else(|| {
        panic!(
            "{}: a creating write must print `id: <ID>` first, got {line:?}",
            SHAPE
        )
    });
    assert!(
        !rest.trim().is_empty(),
        "{}: the creating id line has no id: {line:?}",
        SHAPE
    );
    first_id(rest)
}

/// A write that returns an id the caller knows: the bare id, no `id: ` label.
fn bare(output: &Output) -> String {
    let line = five(output);
    assert!(
        !line.starts_with("id: "),
        "{}: a non-creating write must keep the bare id, got {line:?}",
        SHAPE
    );
    assert!(
        line.split_whitespace().count() == 1,
        "{}: the first line must be the bare id, got {line:?}",
        SHAPE
    );
    line
}

/// The `--json` output is one line whose top-level keys are exactly these.
fn json_keys(output: &Output, extra: &[&str]) {
    assert!(
        output.status.success(),
        "{}: the write failed: {}",
        SHAPE,
        common::stderr(output)
    );
    let text = common::stdout(output);
    let body = text.trim_end_matches('\n');
    assert!(
        !body.contains('\n'),
        "{}: --json prints one line, got {text:?}",
        SHAPE
    );
    let value: Value = serde_json::from_str(body)
        .unwrap_or_else(|error| panic!("{}: --json is not JSON: {error}: {text:?}", SHAPE));
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("{}: --json is not an object: {text:?}", SHAPE));
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    let mut expected = vec!["changed", "id", "missing", "next", "unresolved"];
    expected.extend_from_slice(extra);
    expected.sort_unstable();
    assert_eq!(
        keys, expected,
        "{}: the --json top-level keys changed: {text:?}",
        SHAPE
    );
}

#[test]
fn a_creating_write_prints_id_first() {
    let fx = fixture();
    let criterion = creating(&fx.run(&["criterion", "add", "a criterion"]));
    let need = creating(&fx.run(&["need", "add", "a need", "--targets", &criterion]));
    creating(&fx.run(&[
        "question",
        "add",
        "a question",
        "--decider",
        "master",
        "--options",
        "one",
        "--options",
        "two",
    ]));
    creating(&fx.run(&["decide", "a decision", "--scope-note", "a call"]));
    creating(&fx.run(&["req", "add", "a requirement", "--need", &need]));
    creating(&fx.run(&[
        "req",
        "add",
        "another requirement",
        "--need",
        &need,
        "--ref",
        "https://example/7",
    ]));
}

#[test]
fn a_non_creating_write_keeps_the_bare_id() {
    let fx = fixture();
    let criterion = creating(&fx.run(&["criterion", "add", "a criterion"]));
    let need = creating(&fx.run(&["need", "add", "a need", "--targets", &criterion]));
    let question = creating(&fx.run(&[
        "question",
        "add",
        "a question",
        "--decider",
        "master",
        "--options",
        "one",
        "--options",
        "two",
    ]));
    creating(&fx.run(&["decide", "a decision", "--scope-note", "a call"]));
    let req = creating(&fx.run(&[
        "req",
        "add",
        "a requirement",
        "--need",
        &need,
        "--targets",
        &criterion,
    ]));
    let other = creating(&fx.run(&["req", "add", "another requirement", "--need", &need]));
    let second = creating(&fx.run(&["criterion", "add", "another criterion"]));

    bare(&fx.run(&["link", &need, "targets", &second]));
    bare(&fx.run(&["edit", &criterion, "--reason", "why", "--title", "renamed"]));
    bare(&fx.run(&[
        "question",
        "close",
        &question,
        "--by",
        "fact",
        "--evidence",
        "e",
    ]));
    bare(&fx.run(&[
        "req",
        "approve",
        &req,
        "--design",
        "d1",
        "--heard-by",
        "master",
        "--evidence",
        "e",
    ]));
    bare(&fx.run(&["criterion", "satisfy", &criterion, "--evidence", "e"]));
    bare(&fx.run(&["req", "revise", &req, "--reason", "r", "--source", "s"]));
    bare(&fx.run(&[
        "req",
        "approve",
        &req,
        "--design",
        "d2",
        "--heard-by",
        "master",
        "--evidence",
        "e",
    ]));
    bare(&fx.run(&["req", "done", &req, "--evidence", "shipped"]));
    bare(&fx.run(&["req", "cancel", &other, "--reason", "r", "--source", "s"]));
    bare(&fx.run(&["need", "close", &need, "--by", "fact", "--evidence", "e"]));
}

#[test]
fn json_carries_the_same_fields() {
    let fx = fixture();
    let criterion = creating(&fx.run(&["criterion", "add", "a criterion"]));
    let need = creating(&fx.run(&["need", "add", "a need", "--targets", &criterion]));
    let second = creating(&fx.run(&["criterion", "add", "another criterion"]));

    json_keys(&fx.run(&["--json", "criterion", "add", "one"]), &[]);
    json_keys(
        &fx.run(&["--json", "need", "add", "a need", "--targets", &criterion]),
        &[],
    );
    json_keys(
        &fx.run(&[
            "--json",
            "question",
            "add",
            "a question",
            "--decider",
            "master",
            "--options",
            "one",
            "--options",
            "two",
        ]),
        &[],
    );
    json_keys(
        &fx.run(&["--json", "decide", "a decision", "--scope-note", "s"]),
        &[],
    );
    json_keys(
        &fx.run(&[
            "--json",
            "req",
            "add",
            "a requirement",
            "--need",
            &need,
            "--ref",
            "https://example/9",
        ]),
        &["reference"],
    );
    json_keys(&fx.run(&["--json", "link", &need, "targets", &second]), &[]);
    json_keys(
        &fx.run(&[
            "--json",
            "need",
            "close",
            &need,
            "--by",
            "fact",
            "--evidence",
            "e",
        ]),
        &[],
    );
}
