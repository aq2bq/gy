//! A write that creates a node prints `id: <ID>` as its first line (n-16c8,
//! r-045a): the id is taken with a plain reader, not hunted through the output.
//! A write that returns an id the caller already knows keeps it bare.
use std::path::Path;
use std::process::Command;

fn run(dir: &Path, data: &Path, args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_gy"))
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GY_ACTOR", "test")
        .env("XDG_DATA_HOME", data)
        .output()
        .expect("run gy");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

/// The id a creating write prints on its first line.
fn first_id(text: &str) -> String {
    let line = text.lines().next().unwrap_or("");
    let id = line
        .strip_prefix("id: ")
        .unwrap_or_else(|| panic!("the first line is not an id line: {line:?}"));
    id.split_whitespace().next().unwrap_or("").to_string()
}

#[test]
fn a_creating_write_prints_its_id_first() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let data = tempfile::tempdir().expect("a temp dir");
    std::fs::write(dir.path().join("gy.toml"), "[scopes.a]\n").expect("gy.toml");

    let create = |args: &[&str]| {
        let (ok, text) = run(dir.path(), data.path(), args);
        assert!(ok, "{args:?} failed: {text}");
        first_id(&text)
    };

    let criterion = create(&["--scope", "a", "criterion", "add", "measures one"]);
    let need = create(&[
        "--scope",
        "a",
        "need",
        "add",
        "the need",
        "--targets",
        &criterion,
    ]);
    let question = create(&[
        "--scope",
        "a",
        "question",
        "add",
        "a question",
        "--decider",
        "master",
        "--options",
        "one",
        "--options",
        "two",
    ]);
    let decision = create(&[
        "--scope",
        "a",
        "decide",
        "a decision",
        "--scope-note",
        "a call",
    ]);
    let requirement = create(&[
        "--scope",
        "a",
        "req",
        "add",
        "a requirement",
        "--need",
        &need,
    ]);

    for id in [&criterion, &need, &question, &decision, &requirement] {
        let (ok, text) = run(dir.path(), data.path(), &["show", id]);
        assert!(ok, "gy show {id} failed: {text}");
        assert!(text.contains(id), "gy show {id} did not print it: {text}");
    }

    // A write that returns an id the caller knows keeps the bare line.
    let (ok, closed) = run(
        dir.path(),
        data.path(),
        &[
            "--scope",
            "a",
            "need",
            "close",
            &need,
            "--by",
            "fact",
            "--evidence",
            "done",
        ],
    );
    assert!(ok, "need close failed: {closed}");
    assert!(
        !closed.starts_with("id: "),
        "a close should keep the bare id: {closed:?}"
    );
}
