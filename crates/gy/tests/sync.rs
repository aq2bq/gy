//! The CLI's sync surface (n-6f47 A2, n-8a52 B2): a remote needs git, and a
//! command on a machine with no copy clones it first. The rest is tested
//! against gy-ledger.
mod common;

use common::{fixture, stderr, stdout};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

fn write_toml(fx: &common::Fixture, remote: &str) {
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{remote}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
}

fn git(cwd: &Path, args: &[&str]) {
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
}

fn init_remote(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    let out = Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(dir)
        .output()
        .unwrap();
    assert!(out.status.success());
}

/// The front sync the shared copy needs before a write.
fn front_sync(fx: &common::Fixture) {
    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .arg("sync")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    git(&fx.ledger(), &["config", "user.name", "piko"]);
    git(&fx.ledger(), &["config", "user.email", "piko@example.com"]);
}

fn sync_state(fx: &common::Fixture) -> serde_json::Value {
    let text = std::fs::read_to_string(fx.ledger().join("sync.state")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Wait up to five seconds for the background sync to catch up.
fn wait_for(mut done: impl FnMut() -> bool) -> bool {
    for _ in 0..100 {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn a_write_starts_a_detached_sync() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let fx = fixture();
    write_toml(&fx, &format!("file://{}", remote.display()));
    fx.seed(&[common::criterion("0001", "an ac")]);
    front_sync(&fx);

    let out = fx.run(&["criterion", "add", "another"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        wait_for(|| sync_state(&fx)["last_ok_seq"].as_u64() == Some(2)),
        "the background sync pushed the write"
    );
}

#[test]
fn a_live_pid_stops_a_second_background_sync() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let fx = fixture();
    write_toml(&fx, &format!("file://{}", remote.display()));
    fx.seed(&[common::criterion("0001", "an ac")]);
    front_sync(&fx);
    let before = sync_state(&fx)["last_ok_seq"].as_u64();

    std::fs::write(fx.ledger().join("sync.pid"), std::process::id().to_string()).unwrap();
    let out = fx.run(&["criterion", "add", "another"]);
    assert!(out.status.success(), "{}", stderr(&out));
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(sync_state(&fx)["last_ok_seq"].as_u64(), before);
    std::fs::remove_file(fx.ledger().join("sync.pid")).unwrap();
}

#[test]
fn a_rejected_write_is_noticed_without_touching_the_output() {
    let fx = fixture();
    fx.seed(&[common::need("0001", "a need")]);
    let rejected = serde_json::json!({
        "at": 0,
        "event": {
            "seq": 2, "at": 0, "actor": "piko", "why": "a write", "source": "test",
            "changes": [{"change": "created", "node": "ac-0002",
                         "value": serde_json::to_value(common::criterion("0002", "an ac")).unwrap()}],
        },
        "reason": "the remote changed ac-0002",
    });
    std::fs::write(fx.ledger().join("rejected.jsonl"), format!("{rejected}\n")).unwrap();

    let out = fx.run(&["show", "n-0001"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("a need"));
    assert!(!stdout(&out).contains("notice"), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("notice: your write seq 2"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn handover_reaches_the_remote_first() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let source = fixture();
    write_toml(&source, &format!("file://{}", remote.display()));
    source.seed(&[common::criterion("0001", "from the peer")]);
    front_sync(&source);

    // A machine with no copy reaches the remote through handover, then reads.
    let fresh = fixture();
    write_toml(&fresh, &format!("file://{}", remote.display()));
    let out = fresh.run(&["handover"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = fresh.run(&["list", "--type", "criterion"]);
    assert!(stdout(&out).contains("from the peer"), "{}", stdout(&out));
}

#[test]
fn handover_still_shows_a_copy_when_the_remote_is_unreachable() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let fx = fixture();
    write_toml(&fx, &format!("file://{}", remote.display()));
    fx.seed(&[common::need("0001", "a need")]);
    front_sync(&fx);
    git(
        &fx.ledger(),
        &[
            "remote",
            "set-url",
            "origin",
            "file:///nonexistent/ledger.git",
        ],
    );

    let out = fx.run(&["handover"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("ready needs"), "{}", stdout(&out));
    assert!(wait_for(|| sync_state(&fx)["last_error"].is_string()));
}

#[test]
fn an_unreachable_remote_records_the_error_and_the_write_passes() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let fx = fixture();
    write_toml(&fx, &format!("file://{}", remote.display()));
    fx.seed(&[common::criterion("0001", "an ac")]);
    front_sync(&fx);
    git(
        &fx.ledger(),
        &[
            "remote",
            "set-url",
            "origin",
            "file:///nonexistent/ledger.git",
        ],
    );

    let out = fx.run(&["criterion", "add", "another"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        wait_for(|| sync_state(&fx)["last_error"].is_string()),
        "the background sync recorded the failure"
    );
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
