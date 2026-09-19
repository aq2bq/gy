//! The whole-sync exclusion (n-6b44): a second sync waits, and the pid text
//! and the lock share `sync.pid` without freeing it.
use fs2::FileExt;
use gy_ledger::{FormatVersion, Node, NodeId, NodeKind, format, log, sync};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

fn ident() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        for (key, value) in [
            ("GIT_AUTHOR_NAME", "piko"),
            ("GIT_AUTHOR_EMAIL", "piko@example.com"),
            ("GIT_COMMITTER_NAME", "piko"),
            ("GIT_COMMITTER_EMAIL", "piko@example.com"),
        ] {
            std::env::set_var(key, value);
        }
    });
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

fn url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        "an ac",
    )
    .unwrap()
}

fn event(seq: u64) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: "piko".to_string(),
        why: "test".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: None,
        by_mail: None,
        changes: vec![log::Change::Created {
            node: criterion("0001").id().to_string(),
            value: serde_json::to_value(criterion("0001")).unwrap(),
        }],
    }
}

fn ledger(dir: &Path, events: &[log::Event]) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    for event in events {
        log::append(dir, event).unwrap();
    }
}

fn pair(temp: &Path) -> (PathBuf, PathBuf) {
    let first = temp.join("first");
    let remote = temp.join("remote.git");
    ledger(&first, &[event(1)]);
    bare(&remote);
    sync(&first, &url(&remote)).unwrap();
    (first, remote)
}

/// Hold `sync.pid`'s lock, as a running sync does.
fn hold_pid(dir: &Path) -> std::fs::File {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("sync.pid"))
        .unwrap();
    file.lock_exclusive().unwrap();
    file
}

/// Run sync in another thread, with a deadline; `None` means it is still
/// waiting.
fn running_sync(
    ledger: &Path,
    remote: &str,
) -> (
    std::sync::mpsc::Receiver<gy_ledger::Result<gy_ledger::Sync>>,
    std::thread::JoinHandle<()>,
) {
    let ledger = ledger.to_path_buf();
    let remote = remote.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let _ = tx.send(sync(&ledger, &remote));
    });
    (rx, handle)
}

#[test]
fn a_second_sync_waits_and_a_pid_write_does_not_free_the_lock() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, remote) = pair(temp.path());
    let held = hold_pid(&first);

    let (rx, handle) = running_sync(&first, &url(&remote));
    assert!(
        rx.recv_timeout(Duration::from_millis(300)).is_err(),
        "a second sync waits for the whole-sync lock"
    );
    // The pid text shares the file; rewriting it must not free the lock.
    std::fs::write(first.join("sync.pid"), "123456").unwrap();
    assert!(
        rx.recv_timeout(Duration::from_millis(300)).is_err(),
        "the lock is still held after the pid text is written"
    );

    drop(held);
    let report = rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    assert_eq!(report.seq, 1);
    handle.join().unwrap();
}
