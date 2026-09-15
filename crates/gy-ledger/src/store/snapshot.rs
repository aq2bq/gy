//! The snapshot: a derived file with the current nodes and sequence. It is
//! rewritten on every commit and may be deleted; the log alone restores the
//! same result (D-82).
use super::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

/// The snapshot file name.
pub const FILE: &str = "snapshot.json";

#[derive(Debug, Serialize, Deserialize)]
struct Snapshot {
    seq: u64,
    nodes: BTreeMap<String, Value>,
}

/// Read the snapshot. A missing or unreadable file is `None`, so the caller
/// falls back to replaying the whole log.
pub fn read(dir: &Path) -> Option<(u64, BTreeMap<String, Value>)> {
    let text = std::fs::read_to_string(dir.join(FILE)).ok()?;
    let snapshot: Snapshot = serde_json::from_str(&text).ok()?;
    Some((snapshot.seq, snapshot.nodes))
}

/// Write the snapshot atomically (a temporary file, then a rename).
pub fn write(dir: &Path, seq: u64, nodes: &BTreeMap<String, Value>) -> Result<()> {
    let snapshot = Snapshot {
        seq,
        nodes: nodes.clone(),
    };
    let text = serde_json::to_string(&snapshot).map_err(|e| Error::invalid(e.to_string()))?;
    let temporary = dir.join(".snapshot.tmp");
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, dir.join(FILE))?;
    Ok(())
}
