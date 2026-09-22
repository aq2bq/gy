//! `gy serve`'s sync log (n-08ae, ac-11c9): a failure line and its way out.
//! Local remotes only; a copy is made first, then its origin is broken.
mod common;
use common::{fixture, stderr};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

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

/// Collect the child's standard error until it holds a `sync failed: ` line and
/// the `  → ` line after it, or until `deadline` has passed (n-a237).
fn failure_lines(child: &mut std::process::Child, deadline: Duration) -> String {
    use std::io::{BufRead, BufReader};
    let stderr = child.stderr.take().expect("the child's stderr is piped");
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let started = std::time::Instant::now();
    let mut seen = String::new();
    while started.elapsed() < deadline {
        if let Ok(line) = receiver.recv_timeout(Duration::from_millis(200)) {
            seen.push_str(&line);
            seen.push('\n');
        }
        if seen.contains("sync failed: ") && seen.contains("  → ") {
            break;
        }
    }
    seen
}

/// The first sync: it makes the copy and pushes the seed, as `gy remote sync` does.
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
}

#[test]
fn serve_logs_the_failure_and_its_way_out() {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    init_remote(&remote);
    let fx = fixture();
    write_toml(&fx, &format!("file://{}", remote.display()));
    fx.seed(&[common::criterion("0001", "an ac")]);
    front_sync(&fx);

    // Break the copy's origin so the next round fails fast and offline.
    let out = Command::new("git")
        .current_dir(fx.ledger())
        .args([
            "remote",
            "set-url",
            "origin",
            "file:///nonexistent/ledger.git",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let mut child = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .arg("serve")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Wait for the event, not for a span of time (n-a237): read the child's
    // standard error until the failure and its way out have both appeared, or
    // a generous deadline passes on a loaded machine.
    let err = failure_lines(&mut child, Duration::from_secs(30));
    let _ = child.kill();
    let _ = child.wait();

    assert!(err.contains("sync failed: "), "{err}");
    assert!(
        err.contains("  → "),
        "the way out is on the next line: {err}"
    );
}
