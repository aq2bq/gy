//! The append-only event log: one line per transaction (D-82). A line is
//! `{seq, at, actor, why, source, changes}`; `changes` holds one entry per
//! node, and `value` is the serialized node (opaque to the store).
use super::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The log file name.
pub const FILE: &str = "events.jsonl";

/// The change kinds a line may carry.
pub const CHANGES: [&str; 3] = ["created", "updated", "deleted"];

/// One node change: its id, the kind of change, and the serialized node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub node: String,
    pub change: String,
    pub value: serde_json::Value,
}

/// One transaction: when, who, why, from where, and the node changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub seq: u64,
    pub at: u64,
    pub actor: String,
    pub why: String,
    pub source: String,
    pub changes: Vec<Change>,
}

/// Read the complete events. An incomplete trailing line (a crash before the
/// newline) is discarded, and the byte length of the complete prefix is
/// returned so the caller can truncate it. A complete line that is not an
/// event, or an unknown change kind, is an error (AC-53).
pub fn read(dir: &Path) -> Result<(Vec<Event>, u64)> {
    let bytes = match std::fs::read(dir.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), 0)),
        Err(error) => return Err(error.into()),
    };
    let complete = bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |last| last + 1);
    let mut events = Vec::new();
    for line in String::from_utf8_lossy(&bytes[..complete]).lines() {
        if line.trim().is_empty() {
            continue;
        }
        let event: Event = serde_json::from_str(line)
            .map_err(|_| Error::invalid("the event log has a line that is not an event"))?;
        if event
            .changes
            .iter()
            .any(|change| !CHANGES.contains(&change.change.as_str()))
        {
            return Err(Error::invalid("the event log has an unknown change kind"));
        }
        events.push(event);
    }
    Ok((events, complete as u64))
}

/// Drop the incomplete trailing bytes, if any.
pub fn truncate(dir: &Path, len: u64) -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(dir.join(FILE))?;
    file.set_len(len)?;
    Ok(())
}

/// Append one event as a line and fsync it (AC-45).
pub fn append(dir: &Path, event: &Event) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(FILE))?;
    let line = serde_json::to_string(event).map_err(|e| Error::invalid(e.to_string()))?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
