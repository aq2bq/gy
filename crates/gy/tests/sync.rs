//! The CLI's sync surface (n-6f47 A2): a remote needs git, and says so when it
//! is missing. The happy path is tested against gy-ledger's `sync`.
mod common;

use common::{fixture, stderr};
use std::process::Command;

#[test]
fn sync_without_git_says_git_is_required() {
    let fx = fixture();
    std::fs::write(
        fx.root.join("gy.toml"),
        "remote = \"file:///nonexistent/ledger.git\"\n\n[scopes.a]\n",
    )
    .unwrap();
    fx.seed(&[common::criterion("0001", "an ac")]);

    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .env("PATH", "")
        .arg("sync")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("git is required for a remote"),
        "{}",
        stderr(&out)
    );
}
