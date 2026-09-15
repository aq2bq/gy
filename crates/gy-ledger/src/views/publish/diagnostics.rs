//! The diagnostics at this point: the handover errors and warnings, each with
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
        "エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。\
         警告は注意が要る状態で、`handover` に一覧が出る。\n\n",
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
