//! How a write's writer reads. The human before the agent, when the copy
//! records one (n-d36d), and every name the log was written under (d-b02d).
use crate::ops::repository::{Repository, Store};
use std::collections::BTreeSet;

/// How a write's writer reads: the human before the agent, when the copy
/// records one (n-d36d).
pub(super) fn writer(by: Option<&str>, actor: &str) -> String {
    match by {
        Some(by) => format!("{by} / {actor}"),
        None => actor.to_string(),
    }
}

/// Every name the log was written under, once for the view; `by`, the git name,
/// is not a writer (d-b02d).
pub(super) fn writers<S: Store>(repo: &Repository<S>) -> BTreeSet<String> {
    repo.store()
        .history()
        .iter()
        .map(|entry| entry.actor.name().to_string())
        .collect()
}
