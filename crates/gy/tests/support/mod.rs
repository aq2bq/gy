use std::{fs, process::Command};

/// A lint-clean ledger with one active requirement and its handover fields.
pub fn baseline(scope: &str, parent_issue: Option<u64>) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let store = gy_core::Store::init(temp.path(), scope, parent_issue).unwrap();
    let path = store.root.join(scope).join("requirements/1.md");
    let mut node =
        gy_core::Node::parse(include_str!("../fixtures/requirement.md"), path.clone()).unwrap();
    node.put("scope", scope);
    if let Some(issue) = parent_issue {
        node.put("parent_issue", issue);
    }
    fs::write(path, node.markdown().unwrap()).unwrap();
    drop(store);
    let output = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(temp.path())
        .args(["lint", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "baseline lint: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    temp
}
