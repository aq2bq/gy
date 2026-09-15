use gy_ledger::config;
use std::path::Path;

#[test]
fn a_scope_only_gy_toml_reads() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("gy.toml"),
        "output = \"out/README.md\"\n\n[scopes.a]\n",
    )
    .unwrap();
    let parsed = config::read(temp.path()).unwrap();
    assert!(parsed.scopes.contains_key("a"));
    assert_eq!(parsed.output.as_deref(), Some("out/README.md"));
}

#[test]
fn an_old_gy_toml_is_rejected_by_key() {
    let temp = tempfile::tempdir().unwrap();
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.local/roadmap/kokopelli-gy.toml.snapshot");
    let text = std::fs::read_to_string(snapshot)
        .unwrap_or_else(|_| "[scopes.a]\nparent_issue = 1\n[lint]\n".to_string());
    std::fs::write(temp.path().join("gy.toml"), text).unwrap();
    let error = config::read(temp.path()).unwrap_err().to_string();
    assert!(error.contains("parent_issue"), "{error}");
}
