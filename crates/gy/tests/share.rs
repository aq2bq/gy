//! `gy share` at the command line (n-57c5, ac-efd7): the prefixes, the exit
//! codes, and the gy.toml state. Local `file://` remotes only; no network.
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

fn git_output(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Run `gy share` with the git identity a first commit needs.
fn share(fx: &common::Fixture, url: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .args(["share", url])
        .output()
        .unwrap()
}

fn write_toml(fx: &common::Fixture, remote: &str) {
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{remote}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
}

#[test]
fn share_checks_writes_and_uploads() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let url = format!("file://{}", remote.display());
    let fx = fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);

    let out = share(&fx, &url);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    let first = text.lines().next().unwrap();
    assert!(first.starts_with("checked: "), "{first}");
    assert!(text.contains("wrote: remote"), "{text}");
    assert!(text.contains("uploaded: seq 1"), "{text}");
    assert!(text.contains("invite: "), "{text}");
    assert!(
        std::fs::read_to_string(fx.root.join("gy.toml"))
            .unwrap()
            .contains(&url)
    );
    assert!(fx.ledger().join("remote").is_file());
    assert!(
        git_output(&remote, &["ls-tree", "-r", "--name-only", "main"]).contains("events.jsonl")
    );

    let again = share(&fx, &url);
    assert!(again.status.success(), "{}", stderr(&again));
    assert!(
        stdout(&again).starts_with("checked: already shared with"),
        "{}",
        stdout(&again)
    );
}

#[test]
fn share_refuses_a_remote_it_cannot_use() {
    let fx = fixture();
    write_toml(&fx, "file:///one/ledger.git");
    let different = fx.run(&["share", "file:///two/ledger.git"]);
    assert_eq!(different.status.code(), Some(2));
    assert!(
        stderr(&different).contains("does not switch remotes"),
        "{}",
        stderr(&different)
    );

    let no_copy = fx.run(&["share", "file:///one/ledger.git"]);
    assert_eq!(no_copy.status.code(), Some(2));
    assert!(
        stderr(&no_copy).contains("run gy join"),
        "{}",
        stderr(&no_copy)
    );
}

#[test]
fn share_without_a_ledger_writes_gy_toml_only() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let url = format!("file://{}", remote.display());
    let fx = fixture();

    let out = share(&fx, &url);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("no ledger yet"), "{}", stdout(&out));
    assert!(
        std::fs::read_to_string(fx.root.join("gy.toml"))
            .unwrap()
            .contains(&url)
    );
    assert!(!fx.ledger().join("remote").exists());
}
