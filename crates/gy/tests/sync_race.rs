//! Writes typed in a row, each detaching a background sync, must not make gy
//! refuse a write that landed (n-6b44, ac-e87f). Local bare remotes only.
mod common;
use common::{fixture, stderr};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn init_remote(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    let out = Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(dir)
        .output()
        .unwrap();
    assert!(out.status.success());
}

fn write_toml(fx: &common::Fixture, remote: &str) {
    std::fs::write(
        fx.root.join("gy.toml"),
        format!("remote = \"{remote}\"\n\n[scopes.a]\n"),
    )
    .unwrap();
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The explicit sync that settles behind any detached background sync: it
/// waits for the copy's whole-sync lock, so when it returns nothing is
/// pending.
fn front_sync(fx: &common::Fixture) {
    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .env("GIT_AUTHOR_NAME", "piko")
        .env("GIT_AUTHOR_EMAIL", "piko@example.com")
        .env("GIT_COMMITTER_NAME", "piko")
        .env("GIT_COMMITTER_EMAIL", "piko@example.com")
        .arg("remote")
        .arg("sync")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    git(&fx.ledger(), &["config", "user.name", "piko"]);
    git(&fx.ledger(), &["config", "user.email", "piko@example.com"]);
}

#[test]
fn three_quick_writes_do_not_refuse_a_landed_write() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let fx = fixture();
    write_toml(&fx, &format!("file://{}", remote.display()));
    fx.seed(&[common::criterion("0001", "seed")]);
    front_sync(&fx);

    let started = Instant::now();
    let rounds = 10;
    for round in 0..rounds {
        for i in 0..3 {
            let out = fx.run(&["criterion", "add", &format!("w{round}x{i}")]);
            assert!(out.status.success(), "round {round}: {}", stderr(&out));
        }
        // Settle: this waits for whichever background sync holds the lock.
        front_sync(&fx);
        assert!(
            !fx.ledger().join("rejected.jsonl").exists(),
            "round {round}: rejected.jsonl was written"
        );
        // The remote and this copy carry the same events.
        let local = std::fs::read_to_string(fx.ledger().join("events.jsonl")).unwrap();
        let remote_text = git(&fx.ledger(), &["show", "origin/main:events.jsonl"]);
        assert_eq!(
            local, remote_text,
            "round {round}: the remote and the copy events differ"
        );
        // A following read carries no did-not-land notice.
        let out = fx.run(&["show", "ac-0001"]);
        assert!(out.status.success(), "round {round}: {}", stderr(&out));
        assert!(
            !stderr(&out).contains("did not land"),
            "round {round}: {}",
            stderr(&out)
        );
    }
    eprintln!(
        "three_quick_writes_do_not_refuse_a_landed_write: {rounds} rounds in {:?}",
        started.elapsed()
    );
}
