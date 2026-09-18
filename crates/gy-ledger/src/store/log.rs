//! The append-only event log: one line per transaction (D-82). A line is
//! `{seq, at, actor, why, source, changes}`; a change is a node created,
//! updated, or deleted, or a scope renamed in every node that carries it
//! (n-ff2b).
use super::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

/// The log file name.
pub const FILE: &str = "events.jsonl";

/// One change in a transaction. A node change carries the id and the serialized
/// node; a scope rename carries the names and how many nodes it moved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "kebab-case")]
pub enum Change {
    Created {
        node: String,
        value: Value,
    },
    Updated {
        node: String,
        value: Value,
    },
    Deleted {
        node: String,
        value: Value,
    },
    ScopeRenamed {
        from: String,
        to: String,
        nodes: usize,
    },
}

/// One transaction: when, who, why, from where, and the changes.
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
/// returned so the caller can truncate it. A complete line that is not an event
/// is an error (AC-53).
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
    let mut line = serde_json::to_vec(event).map_err(|e| Error::invalid(e.to_string()))?;
    line.push(b'\n');
    file.write_all(&line)?;
    file.sync_all()?;
    Ok(())
}
