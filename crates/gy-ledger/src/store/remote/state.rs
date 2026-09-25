//! `sync.state`: the copy's last successful sync and last error (n-ecbf,
//! ac-647f). The store writes it; views and the CLI read it through the store.
use super::super::{Error, Result, SyncStatus};
use super::git;
use super::report::Sync;
use fs2::FileExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// True while this process is waiting for the whole-sync lock (n-8b91). A
/// timeout then is not its own failure: the holder is the one whose result
/// tells the truth, so `record_timeout` writes nothing for a waiter.
static WAITING: AtomicBool = AtomicBool::new(false);

/// The whole-sync exclusion (n-6b44): one sync runs at a time per copy, so two
/// syncs cannot both fetch a stale remote ref and push from it. It is a
/// different file from `lock`, which is held only around a change, so a write
/// still never waits on a fetch. `sync.pid` carries the copy's sync pid and is
/// already in the ledger's `.gitignore`; the pid text and this lock share it.
pub(super) struct SyncLock {
    _file: std::fs::File,
}

impl SyncLock {
    /// Wait for the copy's sync lock and hold it for the whole sync.
    pub(super) fn take(ledger: &Path) -> Result<Self> {
        std::fs::create_dir_all(ledger)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(ledger.join("sync.pid"))?;
        WAITING.store(true, Ordering::SeqCst);
        let locked = file.lock_exclusive();
        WAITING.store(false, Ordering::SeqCst);
        locked?;
        Ok(Self { _file: file })
    }
}

/// Record that a background sync gave up after its deadline (n-ecbf). The
/// child calls this from its watchdog, so the next command can report it.
pub fn record_timeout(ledger: &Path) {
    record_timeout_after(ledger, 30);
}

/// The stage a running sync is in, beside the log so a killed child leaves it
/// behind (n-08ae). It holds one word: fetch, rebase, push, or clone.
const STEP_FILE: &str = "sync.step";

/// Record the stage a sync has entered (n-08ae).
pub(super) fn set_step(ledger: &Path, step: &str) {
    let _ = std::fs::write(ledger.join(STEP_FILE), step);
}

/// Forget the stage: the sync ended, one way or the other (n-08ae).
pub fn clear_step(ledger: &Path) {
    let _ = std::fs::remove_file(ledger.join(STEP_FILE));
}

/// The stage a killed sync left, taken so it is never read twice (n-08ae).
fn take_step(ledger: &Path) -> Option<String> {
    let step = std::fs::read_to_string(ledger.join(STEP_FILE)).ok()?;
    let _ = std::fs::remove_file(ledger.join(STEP_FILE));
    let step = step.trim().to_string();
    if step.is_empty() { None } else { Some(step) }
}

/// The last error as `(first line, gy's way out)`: the advice is gy's added
/// last line, so a multi-line git message keeps only its own first line and
/// drops the middle (n-08ae).
pub fn sync_error(ledger: &Path) -> Option<(String, Option<String>)> {
    let error = read_state(ledger).last_error?;
    let mut lines = error.lines();
    let first = lines.next().unwrap_or("").to_string();
    let advice = lines
        .next_back()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string);
    Some((first, advice))
}

/// Record a failure that happened before the sync body ran (n-9f9d): a missing
/// remote in gy.toml, or a refused marker. The next serve tick reads this
/// instead of falling back to the bare `sync failed`.
pub fn record_error(ledger: &Path, message: &str) {
    if !git::is_repo(ledger) {
        return;
    }
    let mut state = read_state(ledger);
    state.last_error = Some(message.to_string());
    state.last_error_at = Some(now());
    let _ = write_state(ledger, &state);
}

/// Record a give-up after `secs` seconds, named for the caller (n-ecbf). When
/// the child left a stage, the message names it (n-08ae). A process that was
/// only waiting for the lock does not write: its timeout is not a failure, and
/// the holder's own result is what the copy keeps (n-8b91).
pub fn record_timeout_after(ledger: &Path, secs: u64) {
    if WAITING.load(Ordering::SeqCst) {
        return;
    }
    if !git::is_repo(ledger) {
        return;
    }
    let mut state = read_state(ledger);
    state.last_error = Some(gave_up(secs, take_step(ledger).as_deref()));
    state.last_error_at = Some(now());
    let _ = write_state(ledger, &state);
}

/// The give-up message: the stage when there is one, the old wording otherwise.
fn gave_up(secs: u64, step: Option<&str>) -> String {
    match step.map(gerund) {
        Some(ing) => format!("the sync gave up after {secs} seconds while {ing} the remote"),
        None => format!("the sync gave up after {secs} seconds waiting for the remote"),
    }
}

/// A stage name as the message reads it: `fetch` -> `fetching`.
fn gerund(step: &str) -> String {
    match step {
        "fetch" => "fetching".to_string(),
        "rebase" => "rebasing".to_string(),
        "push" => "pushing".to_string(),
        "clone" => "cloning".to_string(),
        other => other.to_string(),
    }
}

/// Write `sync.state` after every sync, front or background (n-ecbf). A
/// success clears the last error; a failure keeps the last success and records
/// the message. A state write never hides the sync's own error.
pub(super) fn record_state(ledger: &Path, outcome: &Result<Sync>) {
    // A refused clone leaves no copy, so there is no state to keep.
    if !git::is_repo(ledger) {
        return;
    }
    let mut state = read_state(ledger);
    match outcome {
        Ok(report) => {
            state.last_ok_at = Some(now());
            state.last_ok_seq = Some(report.seq);
            state.last_error = None;
            state.last_error_at = None;
            // A newer peer stays noticed until the upgrade (n-670a B).
            if report.peer_version.is_some() {
                state.peer_version = report.peer_version.clone();
            }
        }
        Err(error) => {
            state.last_error = Some(error.message.clone());
            state.last_error_at = Some(now());
        }
    }
    let _ = write_state(ledger, &state);
}

/// Whether this copy has synced before, so an empty remote now means it was
/// emptied rather than never filled (n-94bb 3B).
pub(super) fn synced_before(ledger: &Path) -> bool {
    read_state(ledger).last_ok_seq.is_some()
}

fn read_state(ledger: &Path) -> SyncStatus {
    std::fs::read_to_string(ledger.join("sync.state"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Replace `sync.state` atomically, so a reader never sees half a file.
fn write_state(ledger: &Path, state: &SyncStatus) -> Result<()> {
    let text = serde_json::to_string(state).map_err(|error| Error::invalid(error.to_string()))?;
    let temporary = ledger.join(format!(".sync.{}.tmp", std::process::id()));
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, ledger.join("sync.state"))?;
    Ok(())
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
