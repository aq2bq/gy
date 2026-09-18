//! The CLI's sync surface (n-6f47 A2, n-8a52 B2): a remote needs git, and a
//! command on a machine with no copy clones it first. The rest is tested
//! against gy-ledger.
mod common;

use common::{fixture, stderr, stdout};
use std::process::Command;

fn write_toml(fx: &common::Fixture, remote: &str) {
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{remote}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
}

#[test]
fn sync_without_git_says_git_is_required() {
    let fx = fixture();
    write_toml(&fx, "file:///nonexistent/ledger.git");
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

#[test]
fn the_first_read_clones_the_copy() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    let bare = Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(&remote)
        .output()
        .unwrap();
    assert!(
        bare.status.success(),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );
    let url = format!("file://{}", remote.display());

    let source = fixture();
    write_toml(&source, &url);
    source.seed(&[common::criterion("0001", "an ac")]);
    let synced = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&source.root)
        .env("XDG_DATA_HOME", &source.data)
        .env("GY_ACTOR", "piko")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .arg("sync")
        .output()
        .unwrap();
    assert!(synced.status.success(), "{}", stderr(&synced));

    // A machine with no copy reads the remote's ledger through the clone.
    let fresh = fixture();
    write_toml(&fresh, &url);
    let out = fresh.run(&["show", "ac-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("an ac"), "{}", stdout(&out));
}
