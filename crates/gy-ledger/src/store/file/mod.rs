//! The file-backed store: an append-only JSONL event log with an exclusive
//! lock, sequence-based conflict detection, and replay on open (D-82).
use super::{Actor, FormatVersion, HistoryEntry, Result, format, log, replay, snapshot};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub use super::as_of::open_at;
pub use super::id::unique_hash;

mod tx;

#[derive(Debug)]
struct Staged {
    node: String,
    value: Vec<u8>,
}

/// The file-backed store. Opening replays the log; `commit` appends one line.
pub struct FileStore {
    dir: PathBuf,
    version: FormatVersion,
    actor: Actor,
    seq: u64,
    nodes: BTreeMap<String, Value>,
    staged: Vec<Staged>,
    staged_rename: Option<(String, String, usize)>,
    history: Vec<HistoryEntry>,
    staged_history: Vec<HistoryEntry>,
    salt: u64,
    replayed: usize,
    retries: u32,
    remote: Option<String>,
    migrated: Option<(u32, u32)>,
}
impl FileStore {
    /// Open the ledger directory, reading the actor from `GY_ACTOR`.
    pub fn open(dir: &Path) -> Result<Self> {
        Self::open_with(dir, |key| std::env::var(key).ok())
    }
    /// The same open with an injected actor lookup. The copy's `remote` marker
    /// is read here and never talked to (n-8a52).
    pub fn open_with(dir: &Path, lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let mut version = format::read(dir)?;
        let migrated = if version < FormatVersion::CURRENT {
            Some((version.0, FormatVersion::CURRENT.0))
        } else {
            None
        };
        if version < FormatVersion::CURRENT {
            format::migrate(dir, version, FormatVersion::CURRENT)?;
            version = FormatVersion::CURRENT;
        }
        let actor = Actor::from_lookup(lookup)?;
        let (events, _) = log::read(dir)?;
        let history = replay::history(&events)?;
        let seq = events.last().map_or(0, |event| event.seq);
        let (nodes, replayed, rewrote) = replay::load_nodes(dir, &events);
        if rewrote {
            snapshot::write(dir, seq, &nodes)?;
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            version,
            actor,
            seq,
            nodes,
            staged: Vec::new(),
            staged_rename: None,
            history,
            staged_history: Vec::new(),
            salt: 0,
            replayed,
            retries: 0,
            remote: read_remote(dir),
            migrated,
        })
    }
    /// The remote this copy syncs with, when it carries the marker (n-8a52).
    pub fn remote(&self) -> Option<&str> {
        self.remote.as_deref()
    }
    /// The format versions this open migrated, `(from, to)`, when it did
    /// (n-f8a2). The caller decides how to tell the reader; the store does not
    /// print.
    pub fn migrated(&self) -> Option<(u32, u32)> {
        self.migrated
    }
    /// How many log events the last open replayed: 0 when the snapshot covered
    /// the whole log.
    pub fn replayed(&self) -> usize {
        self.replayed
    }
}

/// The `remote` marker's URL, when this copy has one.
fn read_remote(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join("remote"))
        .ok()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}
