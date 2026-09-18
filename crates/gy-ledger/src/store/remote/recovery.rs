//! Reading what is left of a damaged copy log (n-94bb 3B, ac-7fd0).
use super::super::{Result, log};
use super::git;
use std::path::Path;

/// A damaged copy's message: what cannot be read, what is readable and not
/// pushed, and how to recover. `None` when the log is whole.
pub(super) fn damaged(ledger: &Path) -> Result<Option<String>> {
    let Ok(bytes) = std::fs::read(ledger.join(log::FILE)) else {
        return Ok(None);
    };
    let complete = bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |last| last + 1);
    let mut good = Vec::new();
    let mut bad = 0usize;
    for line in String::from_utf8_lossy(&bytes[..complete]).lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<log::Event>(line) {
            Ok(event) => good.push(event),
            Err(_) => bad += 1,
        }
    }
    // An unterminated tail is normal: a writer drops it under the lock
    // (n-6b71). Only a complete line that is not an event is damage.
    if bad == 0 {
        return Ok(None);
    }
    let unreadable = bad;
    let committed = head_seq(ledger);
    let pending: Vec<String> = good
        .iter()
        .filter(|event| event.seq > committed)
        .map(|event| format!("seq {} ({})", event.seq, event.why))
        .collect();
    let readable = if pending.is_empty() {
        "none".to_string()
    } else {
        pending.join(", ")
    };
    Ok(Some(format!(
        "the copy's log is damaged: {unreadable} writes cannot be read; readable not-pushed writes: {readable}\nkeep this copy and take the ledger again from the remote, then re-apply the readable writes (gy does not delete anything)"
    )))
}

/// The last sequence HEAD carries; the committed part may still read.
fn head_seq(ledger: &Path) -> u64 {
    git::show(ledger, "HEAD", log::FILE)
        .and_then(|text| {
            text.lines()
                .filter_map(|line| serde_json::from_str::<log::Event>(line).ok())
                .next_back()
        })
        .map_or(0, |event| event.seq)
}
