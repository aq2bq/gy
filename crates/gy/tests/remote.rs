//! `gy remote` and the deprecated old names (n-8d0e, ac-91e8): the group's
//! three leaves do what `share` / `join` / `sync` did, and an old name still
//! works with one line on stderr saying so, leaving stdout alone.
mod common;
use common::{fixture, stderr, stdout};
use std::path::Path;
use std::process::Command;

fn init_remote(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    let out = Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(dir)
        .output()
        .unwrap();
    assert!(out.status.success());
}

/// Run gy as a member would, with the git identity a first commit needs.
fn gy(fx: &common::Fixture, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .args(args)
        .output()
        .unwrap()
}

fn help(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .args(args)
        .arg("--help")
        .output()
        .unwrap();
    stdout(&out)
}

/// The command names a help text lists under `Commands:`.
fn commands(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut on = false;
    for line in text.lines() {
        let trimmed = line.trim_end();
        if !line.starts_with(' ') && trimmed.ends_with(':') {
            on = trimmed == "Commands:";
            continue;
        }
        if on && !line.trim().is_empty() {
            names.push(line.split_whitespace().next().unwrap_or("").to_string());
        }
    }
    names
}

/// The old names still do the work, and their first stderr line says to use
/// the group. The new names say nothing extra.
#[test]
fn the_old_names_still_work_and_say_so() {
    let fx = fixture();
    let old = gy(&fx, &["sync"]);
    assert_eq!(old.status.code(), Some(2));
    assert_eq!(
        stderr(&old).lines().next(),
        Some("gy sync is deprecated; use gy remote sync")
    );
    let new = gy(&fx, &["remote", "sync"]);
    assert_eq!(new.status.code(), Some(2));
    assert!(stderr(&new).contains("no remote"), "{}", stderr(&new));
    assert!(!stderr(&new).contains("deprecated"), "{}", stderr(&new));

    let fx = fixture();
    let old = gy(&fx, &["join"]);
    assert_eq!(old.status.code(), Some(2));
    assert_eq!(
        stderr(&old).lines().next(),
        Some("gy join is deprecated; use gy remote join")
    );

    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let url = format!("file://{}", remote.display());
    let fx = fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);
    let out = gy(&fx, &["share", &url]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stderr(&out).lines().next(),
        Some("gy share is deprecated; use gy remote set <URL>")
    );
    assert!(stdout(&out).starts_with("checked: "), "{}", stdout(&out));
    assert!(stdout(&out).contains("uploaded: seq 1"), "{}", stdout(&out));
}

/// The old names are hidden from `--help`; the group and its three leaves show.
#[test]
fn the_group_is_the_only_name_in_help() {
    let root = commands(&help(&[]));
    assert!(root.contains(&"remote".to_string()), "{root:?}");
    for old in ["share", "join", "sync"] {
        assert!(
            !root.contains(&old.to_string()),
            "{old} is still listed: {root:?}"
        );
    }
    let leaves = commands(&help(&["remote"]));
    for leaf in ["set", "join", "sync"] {
        assert!(
            leaves.contains(&leaf.to_string()),
            "{leaf} missing: {leaves:?}"
        );
    }
}

/// `--json`'s stdout does not change for an old name; only stderr gains the
/// deprecation line.
#[test]
fn json_stdout_does_not_change_for_the_old_name() {
    let fx = fixture();
    let old = gy(&fx, &["--json", "sync"]);
    let new = gy(&fx, &["--json", "remote", "sync"]);
    assert_eq!(old.status.code(), Some(2));
    assert_eq!(stdout(&old), stdout(&new));
    assert!(stderr(&old).contains("deprecated"), "{}", stderr(&old));
    assert!(!stderr(&new).contains("deprecated"), "{}", stderr(&new));
}
