//! Derived views: projections that read through the model and ops only, never
//! the store directly (D-76). N-57 brings show; list, next, handover, and
//! publish follow.
mod show;

pub use show::{EdgeLine, Shown, show};
