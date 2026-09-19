//! The git-backed copy of a ledger (n-6f47, d-39f6). The store layer is the
//! only place that knows the remote exists; model, ops, and views do not.
pub mod git;
mod guard;
mod join;
mod origin;
mod prepare;
pub mod rebase;
mod recovery;
pub mod report;
mod rules;
pub mod shape;
mod share;
mod state;
pub mod sync;

pub use join::{Join, check as join_check, notice as join_notice, run as join_run};
pub use prepare::reconcile;
pub use rebase::{clear_rejected, rejected_notices};
pub use report::{Pulled, Range, Sync};
pub use share::{Share, check as share_check, upload as share_upload};
pub use state::{record_timeout, record_timeout_after};
pub use sync::sync;
