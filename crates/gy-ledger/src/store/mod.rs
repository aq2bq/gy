//! The store layer: transactions, ID generation, history, the format version,
//! and where the canonical ledger lives. It uses no other layer (D-76).
mod as_of;
pub mod file;
pub mod format;
pub mod id;
pub mod location;
pub mod log;
mod memory;
mod replay;
mod snapshot;

pub use file::FileStore;
pub use memory::MemoryStore;
use std::env;

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
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::invalid(error.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// The ledger format version, carried from birth and checked when opened (D-77).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormatVersion(pub u32);
impl FormatVersion {
    pub const CURRENT: Self = Self(2);
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

/// What an undo inverted: an ordinary write, or another undo and its sequence
/// (n-162c). The operation turns this into what the caller should know.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoneKind {
    Write,
    Undo { seq: u64 },
}

/// The why an undo transaction carries, so the next undo can tell it from a
/// write (n-162c). The operation passes it; a store reads it back from the log.
pub(crate) const UNDO_WHY: &str = "undo";

/// Whether a transaction with this why and sequence was a write or an undo
/// (n-162c). An undo's why marks it, so a reopened store reads it the same way.
pub(crate) fn undone_kind(why: &str, seq: u64) -> UndoneKind {
    if why == UNDO_WHY {
        UndoneKind::Undo { seq }
    } else {
        UndoneKind::Write
    }
}

/// One appended change: the transaction it belongs to, when, who, which node,
/// what, why, and the source. Every entry of one transaction shares `seq`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub seq: u64,
    pub at: u64,
    pub actor: Actor,
    pub node: String,
    pub what: String,
    pub why: String,
    pub source: String,
}

/// A source of short ID hashes. Minting is the store's job and the ID type is
/// the model's (D-74); a collision mints a longer hash (N-38). The prefix
/// separates kinds and lets the store check the ids already in use.
pub trait IdSource {
    fn next_hash(&mut self, prefix: &str) -> Result<String>;
}

/// Transactional storage. `commit` applies every staged change or, on error,
/// none (AC-45); history records why and from where (AC-46).
pub trait Store: IdSource {
    fn version(&self) -> FormatVersion;
    /// The stored bytes for a node id, or `None`.
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    /// Every stored node id.
    fn keys(&self) -> Vec<String>;
    fn begin(&mut self);
    /// Stage a node value under its key. An empty value removes the node.
    fn stage(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>);
    /// Stage a scope rename for the open transaction (n-ff2b): every node that
    /// carries the old scope moves to the new one, as one log change. Returns
    /// how many nodes it affects.
    fn rename_scope(&mut self, from: &str, to: &str) -> Result<usize>;
    fn commit(&mut self) -> Result<()>;
    fn rollback(&mut self);
    fn record(&mut self, node: &str, what: &str, why: &str, source: &str);
    /// The kind of change a staged value is for a key: `created` for a new id,
    /// `updated` for a known one, `deleted` for an empty value (D-82). `put`
    /// and `commit` share this, so an open repository's history reads the same
    /// as a reopened one's (n-0a82).
    fn what_for(&self, key: &str, removed: bool) -> &'static str {
        if removed {
            "deleted"
        } else if self.get(key).is_some() {
            "updated"
        } else {
            "created"
        }
    }
    fn history(&self) -> &[HistoryEntry];
    /// Invert the last transaction as a new transaction with this why and
    /// source (D-82). Nothing to invert is an error, and whether it inverted a
    /// write or an undo is returned (n-162c).
    fn undo(&mut self, why: &str, source: &str) -> Result<UndoneKind>;
}

/// The 32-bit FNV-1a hash used for short IDs and the location key (D-74, D-82).
pub(crate) fn fnv1a(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

/// The history line one scope rename leaves: `scope renamed <from> → <to> (n
/// nodes)` (n-ff2b).
pub(crate) fn rename_line(from: &str, to: &str, nodes: usize) -> String {
    format!("scope renamed {from} → {to} ({nodes} nodes)")
}
