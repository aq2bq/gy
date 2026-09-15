//! The in-memory implementation for the skeleton and tests.
use super::{Actor, Error, FormatVersion, HistoryEntry, IdSource, Result, Store};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

/// The in-memory implementation for the skeleton and tests.
#[derive(Debug)]
pub struct MemoryStore {
    version: FormatVersion,
    actor: Actor,
    committed: BTreeMap<String, Vec<u8>>,
    staged: Vec<(String, Vec<u8>)>,
    history: Vec<HistoryEntry>,
    staged_history: Vec<HistoryEntry>,
    undo_stack: Vec<Vec<(String, Option<Vec<u8>>)>>,
    seq: u64,
    salt: u64,
}
impl MemoryStore {
    /// Open the store, reading the actor from `GY_ACTOR`.
    pub fn open(version: FormatVersion) -> Result<Self> {
        Ok(Self::with_actor(version, Actor::from_env()?))
    }
    /// The same open with an injected lookup, so a caller can present an
    /// absent `GY_ACTOR` without touching the process environment.
    pub fn open_with(
        version: FormatVersion,
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Self> {
        Ok(Self::with_actor(version, Actor::from_lookup(lookup)?))
    }
    pub fn with_actor(version: FormatVersion, actor: Actor) -> Self {
        Self {
            version,
            actor,
            committed: BTreeMap::new(),
            staged: Vec::new(),
            history: Vec::new(),
            staged_history: Vec::new(),
            undo_stack: Vec::new(),
            seq: 0,
            salt: 0,
        }
    }
    /// Commit on `Ok`, roll back on `Err`, so a half-finished change writes nothing.
    pub fn transaction<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.begin();
        match f(self) {
            Ok(value) => {
                self.commit()?;
                Ok(value)
            }
            Err(error) => {
                self.rollback();
                Err(error)
            }
        }
    }
}
impl Store for MemoryStore {
    fn version(&self) -> FormatVersion {
        self.version
    }
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.committed.get(key).cloned()
    }
    fn keys(&self) -> Vec<String> {
        self.committed.keys().cloned().collect()
    }
    fn begin(&mut self) {
        self.staged.clear();
        self.staged_history.clear();
    }
    fn stage(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.staged.push((key.into(), value.into()));
    }
    fn commit(&mut self) -> Result<()> {
        let mut before = Vec::new();
        for (key, value) in self.staged.drain(..) {
            before.push((key.clone(), self.committed.get(&key).cloned()));
            if value.is_empty() {
                self.committed.remove(&key);
            } else {
                self.committed.insert(key, value);
            }
        }
        if !before.is_empty() {
            self.undo_stack.push(before);
            self.seq += 1;
        }
        for entry in &mut self.staged_history {
            entry.seq = self.seq;
        }
        self.history.append(&mut self.staged_history);
        Ok(())
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
    /// Re-apply the previous value of every node the last commit touched, as a
    /// new transaction so the undo itself is history (D-82).
    fn undo(&mut self, why: &str, source: &str) -> Result<()> {
        let frame = self
            .undo_stack
            .pop()
            .ok_or_else(|| Error::invalid("there is nothing to undo"))?;
        self.begin();
        for (key, previous) in frame {
            self.stage(key.clone(), previous.unwrap_or_default());
            self.record(&key, "undone", why, source);
        }
        self.commit()
    }
}
impl IdSource for MemoryStore {
    fn next_hash(&mut self, prefix: &str) -> Result<String> {
        self.salt += 1;
        let seed = format!("{prefix}:{}:{}", now(), self.salt);
        Ok(format!("{:04x}", super::fnv1a(&seed) & 0xffff))
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
