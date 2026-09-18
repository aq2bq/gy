//! The git-backed copy of a ledger (n-6f47, d-39f6). The store layer is the
//! only place that knows the remote exists; model, ops, and views do not.
pub mod git;
mod guard;
mod prepare;
pub mod report;
pub mod shape;
mod state;
pub mod sync;

pub use prepare::reconcile;
pub use report::{Pulled, Range, Sync};
pub use state::record_timeout;
pub use sync::sync;
