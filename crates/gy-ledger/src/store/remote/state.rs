//! `sync.state`: the copy's last successful sync and last error (n-ecbf,
//! ac-647f). The store writes it; views and the CLI read it through the store.
use super::super::{Error, Result, SyncStatus};
use super::git;
use super::report::Sync;
use std::path::Path;

/// Record that a background sync gave up after its deadline (n-ecbf). The
/// child calls this from its watchdog, so the next command can report it.
pub fn record_timeout(ledger: &Path) {
    record_timeout_after(ledger, 30);
}

/// Record a give-up after `secs` seconds, named for the caller (n-ecbf).
pub fn record_timeout_after(ledger: &Path, secs: u64) {
    if !git::is_repo(ledger) {
        return;
    }
    let mut state = read_state(ledger);
    state.last_error = Some(format!(
        "the sync gave up after {secs} seconds waiting for the remote"
    ));
    state.last_error_at = Some(now());
    let _ = write_state(ledger, &state);
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
        }
        Err(error) => {
            state.last_error = Some(error.message.clone());
            state.last_error_at = Some(now());
        }
    }
    let _ = write_state(ledger, &state);
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
