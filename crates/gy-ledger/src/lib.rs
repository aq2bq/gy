//! gy's new core, split into four layers with dependencies pointing upward
//! only (D-76): `store` (transactions, ID generation, history, format
//! version), then `model` (node identity here; the five types arrive with
//! N-46), then `ops` (operations), then `views` (derived projections).
//!
//! The model will be the only place that reads a node attribute by a string
//! key, and only in one function; every other field is typed.
mod model;
mod ops;
mod store;
mod views;

pub use model::{NodeId, NodeKind};
pub use ops::Intent;
pub use store::{Actor, Error, FormatVersion, HistoryEntry, IdSource, MemoryStore, Result, Store};
pub use views::{View, reads};
