use std::process::Command;

fn gy(p: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(p)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn render_subcommand_is_gone() {
    let temp = tempfile::tempdir().unwrap();
    gy_core::Store::init(temp.path(), "test", None).unwrap();
    let out = gy(temp.path(), &["render"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unrecognized subcommand"), "{stderr}");
}

#[test]
fn init_writes_no_render_table() {
    let temp = tempfile::tempdir().unwrap();
    let store = gy_core::Store::init(temp.path(), "test", None).unwrap();
    let config = std::fs::read_to_string(store.root.join("gy.toml")).unwrap();
    assert!(!config.contains("[render]"), "{config}");
    assert!(config.contains("[lint]"), "{config}");
}

#[test]
fn init_does_not_append_html_output_to_gitignore() {
    let temp = tempfile::tempdir().unwrap();
    let store = gy_core::Store::init(temp.path(), "test", None).unwrap();
    assert!(!store.root.join(".gitignore").exists());
}
