//! The remote history's shape (n-f4cd, ac-af33): every commit between the copy
//! and the remote is gy's, in a straight line, and only appends to the log.
//! Anything else is refused with what broke and how to recover.
use super::super::{Error, Result, log};
use super::{git, shape};
use std::path::Path;

/// Check the commits `HEAD..upstream` before taking them in.
pub fn check(ledger: &Path, branch: &str, upstream: &str) -> Result<()> {
    let remote_sha = git::rev(ledger, upstream).unwrap_or_default();
    let bad = |what: &str| refuse(ledger, branch, &remote_sha, &remote_sha, what);
    if !git::ancestor(ledger, "HEAD", upstream) {
        return Err(bad("not a descendant"));
    }
    let mut expected = super::sync::seq_at(ledger, "HEAD")? + 1;
    let mut format: Option<u32> = None;
    let range = format!("HEAD..{upstream}");
    for (index, sha) in git::rev_list(ledger, &range)?.iter().enumerate() {
        expected = check_commit(
            ledger,
            branch,
            &remote_sha,
            sha,
            index,
            expected,
            &mut format,
        )?;
    }
    if git::deletions(ledger, "HEAD", upstream, log::FILE)? > 0 {
        return Err(bad("rewrites events.jsonl"));
    }
    Ok(())
}

/// One commit in the range: its trailer continues the sequence, and it touches
/// only gy's files. A format-only commit (`Gy-Format`, the `format` file alone,
/// a higher version) is admitted without advancing the sequence (n-96f8).
/// Returns the next expected sequence.
fn check_commit(
    ledger: &Path,
    branch: &str,
    remote_sha: &str,
    sha: &str,
    index: usize,
    expected: u64,
    format: &mut Option<u32>,
) -> Result<u64> {
    let bad = |what: &str| refuse(ledger, branch, remote_sha, sha, what);
    let message = git::message(ledger, sha)?;
    if let Some(target) = format_trailer(&message) {
        if trailer(&message).is_some() {
            return Err(bad("has both a Gy-Format and a Gy-Seq trailer"));
        }
        if target <= format_now(ledger, format) {
            return Err(bad("lowers the format"));
        }
        let files = git::files_changed(ledger, sha)?;
        if files.len() != 1 || files[0] != super::super::format::FILE {
            return Err(bad("a format commit touches more than format"));
        }
        *format = Some(target);
        return Ok(expected);
    }
    let Some((from, to)) = trailer(&message) else {
        return Err(bad("no Gy-Seq trailer"));
    };
    if from != expected || (from != to && index != 0) {
        return Err(bad("no Gy-Seq trailer"));
    }
    for file in git::files_changed(ledger, sha)? {
        if !shape::FILES.contains(&file.as_str()) {
            return Err(bad(&format!("touches {file}")));
        }
    }
    Ok(to + 1)
}

/// The `Gy-Format` trailer of a format-only commit, when present (n-96f8).
fn format_trailer(message: &str) -> Option<u32> {
    message.lines().rev().find_map(|line| {
        line.strip_prefix("Gy-Format: ")
            .and_then(|value| value.trim().parse().ok())
    })
}

/// The format the incoming commits start from: this copy's committed `format`,
/// read once when the first format-only commit appears (n-96f8).
fn format_now(ledger: &Path, format: &mut Option<u32>) -> u32 {
    if let Some(version) = *format {
        return version;
    }
    let version = git::show(ledger, "HEAD", super::super::format::FILE)
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0);
    *format = Some(version);
    version
}

/// Check the relation before pushing: the remote must be an ancestor of the
/// copy, so a fast-forward push is the only move.
pub fn check_push(ledger: &Path, branch: &str, upstream: &str) -> Result<()> {
    if git::ancestor(ledger, upstream, "HEAD") {
        return Ok(());
    }
    let remote_sha = git::rev(ledger, upstream).unwrap_or_default();
    Err(refuse(
        ledger,
        branch,
        &remote_sha,
        &remote_sha,
        "not a descendant",
    ))
}

/// The `Gy-Seq` trailer as `(first, last)`; a single sequence is both.
fn trailer(message: &str) -> Option<(u64, u64)> {
    message.lines().rev().find_map(|line| {
        let value = line.strip_prefix("Gy-Seq: ")?.trim();
        match value.split_once('-') {
            Some((from, to)) => Some((from.parse().ok()?, to.parse().ok()?)),
            None => Some((value.parse().ok()?, value.parse().ok()?)),
        }
    })
}

/// The refusal: what changed, and the two ways to recover. gy itself never
/// force-pushes (ac-af33).
fn refuse(ledger: &Path, branch: &str, remote_sha: &str, sha: &str, what: &str) -> Error {
    let author = git::author(ledger, sha).unwrap_or_default();
    let head_sha = git::rev(ledger, "HEAD").unwrap_or_default();
    let place = ledger.display();
    Error::invalid(format!(
        "the remote history was changed outside gy at {sha} by {author} ({what}); \
to put the remote back on gy's last commit: `git -C {place} push --force-with-lease={branch}:{remote_sha} origin {head_sha}:{branch}`; \
if the change was intended, stop using gy for this ledger (remove `remote` from gy.toml)"
    ))
}
