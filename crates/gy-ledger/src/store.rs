//! The store layer: transactions, ID generation, history, and the format
//! version. It uses no other layer (D-76). Only the in-memory implementation
//! exists here; the on-disk format is decided in N-38.
use std::{
    collections::BTreeMap,
    env,
    time::{SystemTime, UNIX_EPOCH},
};

/// An invalid value is rejected when it is built, not stored (D-75).
#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}
impl Error {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// The ledger format version, carried from birth and checked when opened (D-77).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormatVersion(pub u32);
impl FormatVersion {
    pub const CURRENT: Self = Self(1);
    pub fn supported(self) -> bool {
        self.0 <= Self::CURRENT.0
    }
}

/// The writer of a change. Every write names its actor; an unset or blank
/// `GY_ACTOR` is an error (D-69). `from_env` is the only place it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor(String);
impl Actor {
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|key| env::var(key).ok())
    }
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        Self::new(lookup("GY_ACTOR").unwrap_or_default())
    }
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(Error::invalid(
                "GY_ACTOR is not set; every write names its actor",
            ));
        }
        Ok(Self(name))
    }
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// One appended change: when, who, which node, what, why, and the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub at: u64,
    pub actor: Actor,
    pub node: String,
    pub what: String,
    pub why: String,
    pub source: String,
}

/// A source of short ID hashes. Minting is the store's job and the ID type is
/// the model's (D-74); a collision mints a longer hash, which N-38 implements.
pub trait IdSource {
    fn next_hash(&mut self, kind: &str) -> Result<String>;
}

/// Transactional storage. `commit` applies every staged change or, on error,
/// none (AC-45); history records why and from where (AC-46).
pub trait Store: IdSource {
    fn version(&self) -> FormatVersion;
    fn begin(&mut self);
    fn stage(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>);
    fn commit(&mut self) -> Result<()>;
    fn rollback(&mut self);
    fn record(&mut self, node: &str, what: &str, why: &str, source: &str);
    fn history(&self) -> &[HistoryEntry];
}

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
        Ok(format!("{:04x}", fnv1a(&seed) & 0xffff))
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
fn fnv1a(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}
