//! The diagnostics for one scope: the handover errors and warnings, each with
//! the handover wording that says what is wrong (d-edb0).
use super::super::handover::handover;
use crate::ops::repository::{Repository, Result, Store};
use std::fmt::Write;

pub(super) fn diagnostics<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
) -> Result<String> {
    let report = handover(repository, scope)?;
    let mut out = String::from(
        "Errors are breaks in the ledger's integrity. Read the node with `show <ID>` and fix it \
         with `edit` or `link`. Warnings are states that need attention, listed by `handover`.\n\n",
    );
    let _ = writeln!(out, "errors: {}", report.errors.len());
    for error in &report.errors {
        let _ = writeln!(out, "- {error}");
    }
    let _ = writeln!(out, "warnings: {}", report.warnings.len());
    for warning in &report.warnings {
        let _ = writeln!(out, "- {}: {}", warning.label, warning.count);
    }
    Ok(out)
}
