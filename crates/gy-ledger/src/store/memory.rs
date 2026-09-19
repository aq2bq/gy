//! The in-memory implementation for the skeleton and tests.
use super::id::{id_seed, unique_hash};
use super::{
    Actor, Error, FormatVersion, Gate, HistoryEntry, IdSource, Result, Store, UndoneKind, log,
    undone_kind,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

/// The in-memory implementation for the skeleton and tests.
#[derive(Debug)]
pub struct MemoryStore {
    version: FormatVersion,
    actor: Actor,
    committed: BTreeMap<String, Vec<u8>>,
    staged: Vec<(String, Vec<u8>)>,
    staged_rename: Option<(String, String, usize)>,
    history: Vec<HistoryEntry>,
    staged_history: Vec<HistoryEntry>,
    undo_stack: Vec<Vec<(String, Option<Vec<u8>>)>>,
    seq: u64,
    salt: u64,
    gate: Option<Arc<dyn Gate>>,
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
            staged_rename: None,
            history: Vec::new(),
            staged_history: Vec::new(),
            undo_stack: Vec::new(),
            seq: 0,
            salt: 0,
            gate: None,
        }
    }
    /// The store for a point in the log (n-10e1): the nodes and the history
    /// already replayed. Writing to it stays in memory.
    pub(crate) fn from_replay(
        version: FormatVersion,
        actor: Actor,
        nodes: BTreeMap<String, Value>,
        history: Vec<HistoryEntry>,
    ) -> Result<Self> {
        let mut committed = BTreeMap::new();
        for (key, value) in nodes {
            committed.insert(key, super::replay::encode(&value)?);
        }
        let seq = history.iter().map(|entry| entry.seq).max().unwrap_or(0);
        Ok(Self {
            version,
            actor,
            committed,
            staged: Vec::new(),
            staged_rename: None,
            history,
            staged_history: Vec::new(),
            undo_stack: Vec::new(),
            seq,
            salt: 0,
            gate: None,
        })
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
    /// Refuse the open transaction when the installed gate does (n-557f).
    fn judged(&self) -> Result<()> {
        match &self.gate {
            Some(gate) => gate.admit(&self.before_map(), &self.pending_changes()),
            None => Ok(()),
        }
    }
    /// The committed nodes as JSON, for the gate (n-557f). A test store may
    /// pay this per commit; the file store hands over the map it already has.
    fn before_map(&self) -> BTreeMap<String, Value> {
        self.committed
            .iter()
            .filter_map(|(key, bytes)| Some((key.clone(), serde_json::from_slice(bytes).ok()?)))
            .collect()
    }
    /// The open transaction's changes, for the gate (n-557f).
    fn pending_changes(&self) -> Vec<log::Change> {
        let mut out = Vec::new();
        for (key, value) in &self.staged {
            let removed = value.is_empty();
            let current = if removed {
                self.committed
                    .get(key)
                    .and_then(|bytes| serde_json::from_slice(bytes).ok())
            } else {
                serde_json::from_slice(value).ok()
            };
            let node = key.clone();
            let value = current.unwrap_or(Value::Null);
            out.push(if removed {
                log::Change::Deleted { node, value }
            } else if self.committed.contains_key(key) {
                log::Change::Updated { node, value }
            } else {
                log::Change::Created { node, value }
            });
        }
        if let Some((from, to, nodes)) = &self.staged_rename {
            out.push(log::Change::ScopeRenamed {
                from: from.clone(),
                to: to.clone(),
                nodes: *nodes,
            });
        }
        out
    }
}
impl Store for MemoryStore {
    fn version(&self) -> FormatVersion {
        self.version
    }
    fn set_gate(&mut self, gate: Arc<dyn Gate>) {
        self.gate = Some(gate);
    }
    fn gate(&self) -> Option<&dyn Gate> {
        self.gate.as_deref()
    }
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.committed.get(key).cloned()
    }
    fn keys(&self) -> Vec<String> {
        self.committed.keys().cloned().collect()
    }
    fn begin(&mut self) {
        self.staged.clear();
        self.staged_rename = None;
        self.staged_history.clear();
    }
    fn stage(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.staged.push((key.into(), value.into()));
    }
    /// Stage a scope rename for the open transaction (n-ff2b). Returns how many
    /// committed nodes carry the old scope.
    fn rename_scope(&mut self, from: &str, to: &str) -> Result<usize> {
        let nodes = self
            .committed
            .values()
            .filter(|bytes| scope_of(bytes).as_deref() == Some(from))
            .count();
        self.staged_rename = Some((from.to_string(), to.to_string(), nodes));
        Ok(nodes)
    }
    fn commit(&mut self) -> Result<()> {
        self.judged()?;
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
        if let Some((from, to, _)) = self.staged_rename.take() {
            let mut frame = Vec::new();
            for (key, bytes) in self.committed.iter_mut() {
                let mut value: Value = serde_json::from_slice(bytes)
                    .map_err(|_| Error::invalid("a node is not valid JSON"))?;
                if !super::replay::rename_value(&mut value, &from, &to) {
                    continue;
                }
                frame.push((key.clone(), Some(bytes.clone())));
                *bytes = serde_json::to_vec(&value)
                    .map_err(|_| Error::invalid("a node is not valid JSON"))?;
            }
            if !frame.is_empty() {
                self.undo_stack.push(frame);
                self.seq += 1;
            }
        }
        for entry in &mut self.staged_history {
            entry.seq = self.seq;
        }
        self.history.append(&mut self.staged_history);
        Ok(())
    }
    fn rollback(&mut self) {
        self.staged.clear();
        self.staged_rename = None;
        self.staged_history.clear();
    }
    fn record(&mut self, node: &str, what: &str, why: &str, source: &str) {
        self.staged_history.push(HistoryEntry {
            seq: 0,
            at: now(),
            actor: self.actor.clone(),
            by: None,
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
    fn undo(&mut self, why: &str, source: &str) -> Result<UndoneKind> {
        let frame = self
            .undo_stack
            .pop()
            .ok_or_else(|| Error::invalid("there is nothing to undo"))?;
        let kind = match self.history.last() {
            Some(entry) => undone_kind(&entry.why, entry.seq),
            None => UndoneKind::Write,
        };
        self.begin();
        for (key, previous) in frame {
            self.stage(key.clone(), previous.unwrap_or_default());
            self.record(&key, "undone", why, source);
        }
        self.commit()?;
        Ok(kind)
    }
}
impl IdSource for MemoryStore {
    fn next_hash(&mut self, prefix: &str) -> Result<String> {
        self.salt += 1;
        let used: BTreeSet<String> = self.committed.keys().cloned().collect();
        let seed = id_seed(prefix, used.len(), self.salt);
        unique_hash(prefix, &seed, &used)
    }
}
fn scope_of(bytes: &[u8]) -> Option<String> {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|value| super::replay::scope_of(&value))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
