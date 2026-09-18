//! The git-backed copy of a ledger (n-6f47, d-39f6). The store layer is the
//! only place that knows the remote exists; model, ops, and views do not.
pub mod git;
pub mod shape;
pub mod sync;

pub use sync::{Sync, sync};
