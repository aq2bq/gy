//! The synced copy's state for handover (n-ecbf, ac-647f): how many writes
//! are not pushed, and how the last sync ended. Absent for a local ledger.
use super::super::ops::repository::{Repository, Store, SyncStatus};
use serde::Serialize;
use std::fmt;

/// The one line, and the JSON object, a synced copy adds to handover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SyncRow {
    pub pending: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ok_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ok_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}
impl fmt::Display for SyncRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sync: {} writes not pushed", self.pending)?;
        if let (Some(at), Some(seq)) = (self.last_ok_at, self.last_ok_seq) {
            write!(f, "; remote last reached {} (seq {seq})", epoch_local(at))?;
        }
        if let Some(error) = &self.last_error {
            write!(f, "; last error: {error}")?;
        }
        Ok(())
    }
}

/// The row when there is something to say: unpushed writes, or a last error.
/// A local ledger has neither (ac-7ae3).
pub fn sync_row<S: Store>(repo: &Repository<S>) -> Option<SyncRow> {
    let pending = repo.store().pending();
    let status: Option<SyncStatus> = repo.store().sync_state();
    if pending == 0
        && status
            .as_ref()
            .is_none_or(|state| state.last_error.is_none())
    {
        return None;
    }
    let status = status.unwrap_or_default();
    Some(SyncRow {
        pending,
        last_ok_at: status.last_ok_at,
        last_ok_seq: status.last_ok_seq,
        last_error: status.last_error,
    })
}

/// An epoch instant, in the reader's own time zone (n-b6b6).
fn epoch_local(at: u64) -> String {
    match chrono::DateTime::from_timestamp(at as i64, 0) {
        Some(instant) => instant
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        None => at.to_string(),
    }
}
