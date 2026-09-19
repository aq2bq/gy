//! The wait-side timeout (n-8b91 2): a sync that is only waiting for the
//! whole-sync lock records no give-up, while a timeout that is not waiting is
//! still recorded and a later success clears it.
use fs2::FileExt;
use gy_ledger::{FileStore, FormatVersion, Node, NodeId, NodeKind, Store, format, log, sync};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

/// The lock and the record path share one process, so these tests run one at a
/// time: the waiter's mark is process-wide.
static SERIAL: Mutex<()> = Mutex::new(());

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

fn ledger(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    log::append(dir, &event(1)).unwrap();
}

/// The copy, its clone, and the bare remote.
fn pair(temp: &Path) -> (PathBuf, PathBuf) {
    let first = temp.join("first");
    let remote = temp.join("remote.git");
    ledger(&first);
    bare(&remote);
    sync(&first, &url(&remote)).unwrap();
    (first, remote)
}

fn store(dir: &Path) -> FileStore {
    FileStore::open_with(dir, |_| Some("piko".into())).unwrap()
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

/// The waiter's give-up must not overwrite the holder's truth (n-8b91).
#[test]
fn a_waiter_does_not_record_a_timeout() {
    let _guard = SERIAL.lock().unwrap_or_else(|poison| poison.into_inner());
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, remote) = pair(temp.path());
    let held = hold_pid(&first);

    let (rx, handle) = running_sync(&first, &url(&remote));
    assert!(
        rx.recv_timeout(Duration::from_millis(300)).is_err(),
        "the second sync waits for the whole-sync lock"
    );
    // The waiter reached its deadline while still waiting: it records nothing.
    gy_ledger::record_timeout(&first);
    let state = store(&first).sync_state().unwrap();
    assert!(
        state.last_error.is_none(),
        "a waiter records no give-up: {:?}",
        state.last_error
    );

    drop(held);
    let report = rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    assert_eq!(report.seq, 1);
    handle.join().unwrap();
}

/// A timeout that is not a wait is recorded and reads back, handover included.
#[test]
fn a_timeout_that_is_not_a_wait_is_recorded() {
    let _guard = SERIAL.lock().unwrap_or_else(|poison| poison.into_inner());
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, _remote) = pair(temp.path());

    gy_ledger::record_timeout_after(&first, 7);

    let state = store(&first).sync_state().unwrap();
    assert_eq!(
        state.last_error.as_deref(),
        Some("the sync gave up after 7 seconds waiting for the remote")
    );
    let error = "the sync gave up after 7 seconds waiting for the remote".to_string();
    assert_eq!(gy_ledger::sync_error(&first), Some((error.clone(), None)));
    let report = gy_ledger::handover(&gy_ledger::Repository::new(store(&first)), None).unwrap();
    assert!(
        format!("{report}").contains(&format!("last error: {error}")),
        "handover's sync line carries the timeout: {report}"
    );
}

/// A later success is what the copy keeps, so the timeout is gone.
#[test]
fn a_success_after_a_timeout_clears_it() {
    let _guard = SERIAL.lock().unwrap_or_else(|poison| poison.into_inner());
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, remote) = pair(temp.path());

    gy_ledger::record_timeout_after(&first, 7);
    assert!(store(&first).sync_state().unwrap().last_error.is_some());

    sync(&first, &url(&remote)).unwrap();

    assert!(store(&first).sync_state().unwrap().last_error.is_none());
}
