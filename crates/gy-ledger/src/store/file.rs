//! The file-backed store: an append-only JSONL event log with an exclusive
//! lock, sequence-based conflict detection, and replay on open (D-82).
use super::{
    Actor, Error, FormatVersion, HistoryEntry, IdSource, Result, Store, format, log, replay,
    snapshot,
};
use fs2::FileExt;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Widen the short hash until it is unique among `used` full ids (D-74):
/// four hex digits, then six, then eight.
pub fn unique_hash(prefix: &str, base: u32, used: &BTreeSet<String>) -> Result<String> {
    let hex = format!("{base:08x}");
    for length in [4, 6, 8] {
        let hash = &hex[..length];
        if !used.contains(&format!("{prefix}-{hash}")) {
            return Ok(hash.to_string());
        }
    }
    Err(Error::invalid("no unused id hash remains"))
}

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
    history: Vec<HistoryEntry>,
    staged_history: Vec<HistoryEntry>,
    salt: u64,
    replayed: usize,
}
impl FileStore {
    /// Open the ledger directory, reading the actor from `GY_ACTOR`.
    pub fn open(dir: &Path) -> Result<Self> {
        Self::open_with(dir, |key| std::env::var(key).ok())
    }
    /// The same open with an injected actor lookup.
    pub fn open_with(dir: &Path, lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let version = format::read(dir)?;
        let actor = Actor::from_lookup(lookup)?;
        let (events, complete) = log::read(dir)?;
        let len = std::fs::metadata(dir.join(log::FILE)).map_or(0, |meta| meta.len());
        if len > complete {
            log::truncate(dir, complete)?;
        }
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
            history,
            staged_history: Vec::new(),
            salt: 0,
            replayed,
        })
    }
    /// How many log events the last open replayed: 0 when the snapshot covered
    /// the whole log.
    pub fn replayed(&self) -> usize {
        self.replayed
    }
    /// Turn the staged nodes into log changes, deriving the change kind from
    /// the current nodes: a new id is created, a known id is updated, and an
    /// empty value is a deletion.
    fn build_changes(&self) -> Result<Vec<log::Change>> {
        let mut changes = Vec::new();
        for item in &self.staged {
            let value: Value = if item.value.is_empty() {
                self.nodes.get(&item.node).cloned().unwrap_or(Value::Null)
            } else {
                serde_json::from_slice(&item.value)
                    .map_err(|_| Error::invalid("a staged node is not valid JSON"))?
            };
            let change = if item.value.is_empty() {
                "deleted"
            } else if self.nodes.contains_key(&item.node) {
                "updated"
            } else {
                "created"
            };
            changes.push(log::Change {
                node: item.node.clone(),
                change: change.to_string(),
                value,
            });
        }
        Ok(changes)
    }
    /// Commit on `Ok`, roll back on `Err`, so a half-finished change writes nothing.
    pub fn transaction<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.begin();
        match f(self) {
            Ok(value) => match self.commit() {
                Ok(()) => Ok(value),
                Err(error) => {
                    self.rollback();
                    Err(error)
                }
            },
            Err(error) => {
                self.rollback();
                Err(error)
            }
        }
    }
    /// Append the open transaction under the lock, then update the state.
    fn append(&mut self, why: String, source: String) -> Result<()> {
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.dir.join("lock"))?;
        lock.lock_exclusive()?;
        let (events, _) = log::read(&self.dir)?;
        if events.last().map_or(0, |event| event.seq) != self.seq {
            return Err(Error::invalid(
                "another writer advanced the ledger; reopen and retry",
            ));
        }
        let event = log::Event {
            seq: self.seq + 1,
            at: now(),
            actor: self.actor.name().to_string(),
            why,
            source,
            changes: self.build_changes()?,
        };
        log::append(&self.dir, &event)?;
        self.seq += 1;
        for entry in &mut self.staged_history {
            entry.seq = self.seq;
        }
        replay::apply(&mut self.nodes, &event);
        self.history.append(&mut self.staged_history);
        self.staged.clear();
        snapshot::write(&self.dir, self.seq, &self.nodes)?;
        Ok(())
    }
}
impl Store for FileStore {
    fn version(&self) -> FormatVersion {
        self.version
    }
    fn get(&self, node: &str) -> Option<Vec<u8>> {
        self.nodes
            .get(node)
            .and_then(|value| serde_json::to_vec(value).ok())
    }
    fn keys(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }
    fn begin(&mut self) {
        self.staged.clear();
        self.staged_history.clear();
    }
    fn stage(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.staged.push(Staged {
            node: key.into(),
            value: value.into(),
        });
    }
    fn commit(&mut self) -> Result<()> {
        if self.staged.is_empty() {
            if self.staged_history.is_empty() {
                return Ok(());
            }
            return Err(Error::invalid(
                "nothing staged; a write changes at least one node",
            ));
        }
        let Some(head) = self.staged_history.first() else {
            return Err(Error::invalid("every write names its why and source"));
        };
        self.append(head.why.clone(), head.source.clone())
    }
    fn rollback(&mut self) {
        self.staged.clear();
        self.staged_history.clear();
    }
    fn record(&mut self, node: &str, what: &str, why: &str, source: &str) {
        self.staged_history.push(HistoryEntry {
            seq: 0,
            at: now(),
            actor: self.actor.clone(),
            node: node.into(),
            what: what.into(),
            why: why.into(),
            source: source.into(),
        });
    }
    fn history(&self) -> &[HistoryEntry] {
        &self.history
    }
    /// Append the inverse of the last transaction as a new line (D-82). The
    /// log is never rewritten; the undo is one more transaction with the given
    /// why and source. A created node is deleted, an updated node returns to
    /// its earlier value, and a deleted node comes back.
    fn undo(&mut self, why: &str, source: &str) -> Result<()> {
        let (events, _) = log::read(&self.dir)?;
        let last = events
            .last()
            .ok_or_else(|| Error::invalid("there is nothing to undo"))?;
        let mut before = BTreeMap::new();
        for event in &events[..events.len() - 1] {
            replay::apply(&mut before, event);
        }
        self.begin();
        for change in &last.changes {
            let value = match change.change.as_str() {
                "created" => Vec::new(),
                "updated" => replay::encode(
                    before
                        .get(&change.node)
                        .ok_or_else(|| Error::invalid("the updated node has no earlier value"))?,
                )?,
                "deleted" => replay::encode(&change.value)?,
                _ => return Err(Error::invalid("the log has an unknown change kind")),
            };
            self.stage(change.node.clone(), value);
            self.record(&change.node, change.change.as_str(), why, source);
        }
        self.commit()
    }
}
impl IdSource for FileStore {
    fn next_hash(&mut self, prefix: &str) -> Result<String> {
        self.salt += 1;
        let base = super::fnv1a(&format!("{prefix}:{}:{}", now(), self.salt));
        unique_hash(prefix, base, &self.nodes.keys().cloned().collect())
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
