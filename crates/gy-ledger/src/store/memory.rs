//! The in-memory implementation for the skeleton and tests.
use super::{Actor, FormatVersion, HistoryEntry, IdSource, Result, Store};
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
            salt: 0,
        }
    }
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.committed.get(key).map(Vec::as_slice)
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
    fn begin(&mut self) {
        self.staged.clear();
        self.staged_history.clear();
    }
    fn stage(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.staged.push((key.into(), value.into()));
    }
    fn commit(&mut self) -> Result<()> {
        for (key, value) in self.staged.drain(..) {
            self.committed.insert(key, value);
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
}
impl IdSource for MemoryStore {
    fn next_hash(&mut self, kind: &str) -> Result<String> {
        self.salt += 1;
        let seed = format!("{kind}:{}:{}", now(), self.salt);
        Ok(format!("{:04x}", super::fnv1a(&seed) & 0xffff))
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
