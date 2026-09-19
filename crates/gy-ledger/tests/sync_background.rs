//! The lock stays off communication, `sync.state` is written, and handover
//! shows the synced copy (n-ecbf, ac-647f). Local bare remotes only.
use fs2::FileExt;
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, Operation, Repository, Store, format, handover, log,
    sync,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

mod common;
use common::ident;

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

fn set_user(dir: &Path) {
    git(dir, &["config", "user.name", "piko"]);
    git(dir, &["config", "user.email", "piko@example.com"]);
}

fn ledger(dir: &Path) -> u64 {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    write(dir).unwrap();
    log::read(dir).unwrap().0.last().unwrap().seq
}

fn write(dir: &Path) -> gy_ledger::Result<gy_ledger::Outcome<gy_ledger::NodeId>> {
    let mut repo = Repository::new(FileStore::open_with(dir, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "an ac".into(),
        body: None,
    }
    .run(&mut repo)
}

fn store(dir: &Path) -> FileStore {
    FileStore::open_with(dir, |_| Some("piko".into())).unwrap()
}

/// The copy, its clone, and the bare remote.
fn pair(temp: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let first = temp.join("first");
    let remote = temp.join("remote.git");
    ledger(&first);
    bare(&remote);
    sync(&first, &url(&remote)).unwrap();
    let second = temp.join("second");
    sync(&second, &url(&remote)).unwrap();
    (first, second, remote)
}

/// Hold the ledger's lock, as a concurrent sync would.
fn hold_lock(dir: &Path) -> std::fs::File {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("lock"))
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
fn an_up_to_date_sync_does_not_wait_for_the_lock() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _second, remote) = pair(temp.path());
    let held = hold_lock(&first);

    let (rx, handle) = running_sync(&first, &url(&remote));
    assert!(
        rx.recv_timeout(Duration::from_secs(2)).is_ok(),
        "an up-to-date sync should not take the lock"
    );
    drop(held);
    handle.join().unwrap();
}

#[test]
fn a_pull_waits_for_the_lock() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second, remote) = pair(temp.path());
    set_user(&second);
    write(&second).unwrap();
    sync(&second, &url(&remote)).unwrap();
    let held = hold_lock(&first);

    let (rx, handle) = running_sync(&first, &url(&remote));
    assert!(
        rx.recv_timeout(Duration::from_millis(300)).is_err(),
        "a pull should wait for the lock"
    );
    drop(held);
    let report = rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    assert!(report.pulled.is_some());
    handle.join().unwrap();
}

#[test]
fn sync_state_records_success_and_error() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let remote = temp.path().join("remote.git");
    let seq = ledger(&first);
    bare(&remote);

    sync(&first, &url(&remote)).unwrap();
    let state = store(&first).sync_state().unwrap();
    assert_eq!(state.last_ok_seq, Some(seq));
    assert!(state.last_ok_at.is_some() && state.last_error.is_none());

    // An unreachable remote leaves the last success and records the error.
    git(
        &first,
        &[
            "remote",
            "set-url",
            "origin",
            "file:///nonexistent/ledger.git",
        ],
    );
    sync(&first, &url(&remote)).unwrap_err();
    let state = store(&first).sync_state().unwrap();
    assert_eq!(state.last_ok_seq, Some(seq));
    assert!(state.last_error.is_some() && state.last_error_at.is_some());
}

#[test]
fn the_handover_line_shows_only_the_first_error_line() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _second, _remote) = pair(temp.path());
    std::fs::write(
        first.join("sync.state"),
        serde_json::json!({"last_error": "first line\nsecond line"}).to_string(),
    )
    .unwrap();

    let report = handover(&Repository::new(store(&first)), None).unwrap();
    let line = format!("{report}");
    assert!(line.contains("last error: first line"), "{line}");
    assert!(!line.contains("second line"), "{line}");
    // The state file and --json keep the whole message.
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["sync"]["last_error"], "first line\nsecond line");
}

#[test]
fn a_timeout_is_recorded() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _second, _remote) = pair(temp.path());

    gy_ledger::record_timeout(&first);
    let state = store(&first).sync_state().unwrap();
    assert!(
        state.last_error.unwrap().contains("30 seconds"),
        "the timeout is recorded"
    );
}

#[test]
fn a_timeout_names_the_stage_and_takes_it() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _second, _remote) = pair(temp.path());
    std::fs::write(first.join("sync.step"), "fetch").unwrap();

    gy_ledger::record_timeout_after(&first, 10);

    let state = store(&first).sync_state().unwrap();
    assert_eq!(
        state.last_error.as_deref(),
        Some("the sync gave up after 10 seconds while fetching the remote")
    );
    assert!(!first.join("sync.step").exists(), "the stage is taken");
    assert_eq!(
        gy_ledger::sync_error(&first),
        Some((
            "the sync gave up after 10 seconds while fetching the remote".to_string(),
            None
        ))
    );
}

#[test]
fn sync_error_takes_gys_last_line_as_the_advice() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _second, _remote) = pair(temp.path());
    std::fs::write(
        first.join("sync.state"),
        serde_json::json!({
            "last_error": "remote: Repository not found.\nfatal: could not read from remote\ncheck the remote URL and your git credentials",
        })
        .to_string(),
    )
    .unwrap();

    assert_eq!(
        gy_ledger::sync_error(&first),
        Some((
            "remote: Repository not found.".to_string(),
            Some("check the remote URL and your git credentials".to_string())
        )),
        "git's middle line is dropped"
    );
}

#[test]
fn a_finished_sync_leaves_no_stage() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _second, _remote) = pair(temp.path());
    assert!(!first.join("sync.step").exists(), "the stage is cleared");
}

#[test]
fn handover_shows_pending_and_json_on_a_synced_copy() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _second, _remote) = pair(temp.path());
    set_user(&first);
    write(&first).unwrap();

    assert_eq!(store(&first).pending(), 1);
    let report = handover(&Repository::new(store(&first)), None).unwrap();
    assert_eq!(report.sync.as_ref().unwrap().pending, 1);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["sync"]["pending"], 1);

    // A local ledger has no sync state at all.
    let plain = temp.path().join("plain");
    ledger(&plain);
    assert_eq!(store(&plain).pending(), 0);
    let report = handover(&Repository::new(store(&plain)), None).unwrap();
    assert!(report.sync.is_none());
    let json = serde_json::to_value(&report).unwrap();
    assert!(json.get("sync").is_none());
}
