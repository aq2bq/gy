use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn script() -> PathBuf {
    root().join("scripts/measure.sh")
}

fn measure(dir: &Path, src: &str, tests: &str, json: bool) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(script())
        .arg(src)
        .arg(tests)
        .arg("--base")
        .arg("HEAD")
        .current_dir(dir);
    if json {
        command.arg("--json");
    }
    command.output().unwrap()
}

/// Store is a directory, the other layers are files, with upward-only imports.
fn small_crate(dir: &Path) {
    fs::create_dir_all(dir.join("src/store")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    let files = [
        ("store/mod.rs", "pub fn version() -> u32 {\n    1\n}\n"),
        (
            "model.rs",
            "use crate::store::version;\n\npub fn kind() -> u32 {\n    version()\n}\n",
        ),
        (
            "ops.rs",
            "use crate::{model::kind, store::version};\n\npub fn apply() -> u32 {\n    kind() + version()\n}\n",
        ),
        (
            "views.rs",
            "use crate::ops::apply;\n\npub fn show() -> u32 {\n    apply()\n}\n",
        ),
    ];
    for (name, text) in files {
        fs::write(dir.join("src").join(name), text).unwrap();
    }
}

#[test]
fn gy_core_shows_the_violations() {
    let out = measure(&root(), "crates/gy-core/src", "crates/gy/tests", false);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("operations.rs"), "{text}");
    assert!(text.contains("glob imports: 9"), "{text}");
    assert!(text.contains("crates/gy/tests/workflows.rs"), "{text}");
}

#[test]
fn json_reports_every_metric() {
    let out = measure(&root(), "crates/gy-core/src", "crates/gy/tests", true);
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    for key in [
        "files",
        "functions",
        "globs",
        "string_keys",
        "tests",
        "diff",
        "layers",
        "violations",
    ] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
    assert_eq!(value["globs"]["count"], 9);
    assert!(
        value["files"]["over"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry.as_str().unwrap().contains("operations.rs"))
    );
}

#[test]
fn a_small_upward_crate_passes() {
    let temp = tempfile::tempdir().unwrap();
    small_crate(temp.path());
    let out = measure(temp.path(), "src", "tests", false);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn a_downward_use_is_a_violation() {
    let temp = tempfile::tempdir().unwrap();
    small_crate(temp.path());
    let views = temp.path().join("src/views.rs");
    let mut text = fs::read_to_string(&views).unwrap();
    text.push_str("use crate::store::version;\n");
    fs::write(&views, text).unwrap();
    let out = measure(temp.path(), "src", "tests", false);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("violations"));
}
