//! A missing gy.toml says how to start one (n-1de7): the failure names the
//! directory to pass and the one line that begins a repository.
mod common;

use common::{fixture, stderr};

#[test]
fn a_missing_gy_toml_says_how_to_start_one() {
    let fx = fixture();
    std::fs::remove_file(fx.root.join("gy.toml")).unwrap();
    let out = fx.run(&["handover"]);
    assert_eq!(out.status.code(), Some(2));
    let text = stderr(&out);
    assert!(text.contains("no gy.toml"), "{text}");
    assert!(text.contains("-C"), "{text}");
    assert!(text.contains("[scopes."), "{text}");
}
