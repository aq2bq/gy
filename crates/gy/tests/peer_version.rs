//! A peer writing with a newer gy (n-670a B, r-3f07): the pulled
//! `Gy-Version` is noticed on sync and handover, and silence when versions
//! match. Copies share one file:// remote; a local-only clone supplies the
//! peer's hand-written version without racing a detached sync.
mod common;
use common::{fixture, stderr, stdout};
use std::path::Path;
use std::process::Command;

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}\nstdout: {}\nstderr: {}",
        out.status,
        stdout(&out),
        stderr(&out)
    );
}

fn commit_with_version(fx: &common::Fixture, version: &str, seq: u64) {
    // pair() already synced the initial record. Prepare the next record in
    // its own clone: no remote in gy.toml means no detached sync can commit
    // it before this test supplies the version trailer.
    let remote = std::fs::read_to_string(fx.ledger().join("remote")).unwrap();
    let peer = fixture();
    let ledger = peer.ledger();
    std::fs::create_dir_all(&ledger).unwrap();
    git(&ledger, &["clone", "-q", "--branch", "main", remote.trim(), "."]);
    let out = run(&peer, &["criterion", "add", "peer ac"]);
    assert!(out.status.success(), "{}", stderr(&out));

    git(&ledger, &["add", "events.jsonl"]);
    let message = format!("hand\n\nactor: piko\nby: piko\nGy-Version: {version}\nGy-Seq: {seq}\n");
    git(&ledger, &["commit", "-q", "-m", &message]);
    git(&ledger, &["push", "-q", "origin", "HEAD:main"]);
}

fn run(fx: &common::Fixture, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gy"));
    command
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .args(args);
    command.output().unwrap()
}

fn write_toml(fx: &common::Fixture, remote: &str) {
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{remote}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
}

/// Two copies sharing one bare remote, the first already synced.
fn pair() -> (tempfile::TempDir, common::Fixture, common::Fixture) {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    std::fs::create_dir_all(&remote).unwrap();
    let out = Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(&remote)
        .output()
        .unwrap();
    assert!(out.status.success());
    let (fx1, fx2) = (fixture(), fixture());
    let url = format!("file://{}", remote.display());
    write_toml(&fx1, &url);
    write_toml(&fx2, &url);
    let out = run(&fx1, &["criterion", "add", "an ac"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = run(&fx1, &["remote", "sync"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = run(&fx2, &["remote", "sync"]);
    assert!(out.status.success(), "{}", stderr(&out));
    (temp, fx1, fx2)
}

#[test]
fn newer_peer_is_noticed_on_sync_and_handover() {
    let (_temp, fx1, fx2) = pair();
    commit_with_version(&fx1, "9.9.9", 2);

    let out = run(&fx2, &["remote", "sync"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    let last = text.lines().last().unwrap_or_default();
    assert!(
        last.contains("someone sharing this ledger writes with gy 9.9.9"),
        "{text}"
    );
    assert!(last.contains("cargo install gy --locked"), "{text}");

    let out = run(&fx2, &["handover"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stderr(&out).contains("9.9.9"), "{}", stderr(&out));
}

#[test]
fn newer_peer_keeps_json_stdout_pure() {
    let (_temp, fx1, fx2) = pair();
    commit_with_version(&fx1, "9.9.9", 2);

    let out = run(&fx2, &["remote", "sync", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert!(value.get("peer_version").is_none(), "{value}");
    assert!(stderr(&out).contains("9.9.9"), "{}", stderr(&out));
}

#[test]
fn matching_versions_stay_silent() {
    let (_temp, fx1, fx2) = pair();
    let out = run(&fx1, &["criterion", "add", "peer ac"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = run(&fx1, &["remote", "sync"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let out = run(&fx2, &["remote", "sync"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!stdout(&out).contains("cargo install"), "{}", stdout(&out));
    assert!(!stderr(&out).contains("cargo install"), "{}", stderr(&out));
    let out = run(&fx2, &["handover"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!stderr(&out).contains("cargo install"), "{}", stderr(&out));
}
