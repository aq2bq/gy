//! Sending the copy's writes to the remote (n-6a8d): the push side of `gy
//! sync`, moved out of `sync.rs` with no change in behaviour.
use super::super::{Error, FormatVersion, Result, format, log};
use super::report::{Range, Sync};
use super::sync::{locked, seq_at, working_seq};
use super::{git, guard, shape, state};
use std::path::Path;

/// The version the remote must be raised to: the copy's own, when it is newer
/// than the remote's (n-96f8). A copy without a readable format raises nothing.
pub(super) fn raise_target(local: Option<u32>, remote: Option<u32>) -> Option<u32> {
    match (local, remote) {
        (Some(local), Some(remote)) if local > remote => Some(local),
        _ => None,
    }
}

/// Raise the remote's format to `target` with one commit that changes only
/// `format`, and push it (n-96f8). A refused push rolls the commit back, like a
/// refused write, so the next sync tries again.
pub(super) fn raise_format(ledger: &Path, branch: &str, target: u32) -> Result<()> {
    let before = git::rev(ledger, "HEAD");
    commit_format(ledger, target)?;
    if let Err(error) = git::push_ff(ledger, branch) {
        if let Some(before) = before {
            git::reset_mixed(ledger, &before)?;
        }
        return Err(error);
    }
    Ok(())
}

/// Write the copy's format and commit it alone, so the raise lands before any
/// write of the new rules (n-96f8).
fn commit_format(ledger: &Path, target: u32) -> Result<()> {
    format::write(ledger, FormatVersion(target))?;
    let sha = git::hash_object(ledger, format!("{}\n", target).as_bytes())?;
    git::update_index(ledger, format::FILE, &sha)?;
    git::commit(ledger, &format_message(target))
}

/// The format-only commit's message: the dedicated trailer, never `Gy-Seq`
/// (n-96f8).
fn format_message(target: u32) -> String {
    format!("gy format: raise to {target}\n\nGy-Format: {target}\n")
}

/// Send the copy's commits: the remote must be behind, then push (committing
/// each uncommitted write first).
pub(super) fn push(
    ledger: &Path,
    branch: &str,
    upstream: &str,
    committed: u64,
    report: &mut Sync,
    raise: Option<u32>,
) -> Result<()> {
    guard::check_push(ledger, branch, upstream)?;
    let remote_seq = seq_at(ledger, upstream)?;
    if committed > remote_seq {
        if let Some(target) = raise {
            // Committed writes are already ahead of the remote; the raise can
            // only land after them here (d-bace, lead 2026-09-19).
            commit_format(ledger, target)?;
        }
        git::push_ff(ledger, branch)?;
        report.pushed = Some(Range {
            from: remote_seq + 1,
            to: committed,
        });
    } else {
        locked(ledger, || {
            push_writes(ledger, branch, seq_at(ledger, "HEAD")?, report, raise)
        })?;
    }
    report.seq = working_seq(ledger)?;
    Ok(())
}

/// Commit each uncommitted write as its own commit and push the branch. A
/// failed push rolls the commits back, so HEAD stays behind upstream and the
/// next sync tries again (n-ecbf 2B). When `raise` names a version, the
/// format-only commit comes first, before the writes (n-96f8).
pub(super) fn push_writes(
    ledger: &Path,
    branch: &str,
    committed: u64,
    report: &mut Sync,
    raise: Option<u32>,
) -> Result<()> {
    let before = git::rev(ledger, "HEAD");
    let base = git::show(ledger, "HEAD", log::FILE).unwrap_or_default();
    if let Some(target) = raise {
        commit_format(ledger, target)?;
    }
    let text = std::fs::read_to_string(ledger.join(log::FILE))?;
    let appended = text
        .strip_prefix(&base)
        .ok_or_else(|| Error::invalid("the working log is not HEAD plus writes"))?;
    let mut cumulative = base;
    for line in appended.lines().filter(|line| !line.trim().is_empty()) {
        let event: log::Event = serde_json::from_str(line)
            .map_err(|_| Error::invalid("the working log has a line that is not an event"))?;
        cumulative.push_str(line);
        cumulative.push('\n');
        let sha = git::hash_object(ledger, cumulative.as_bytes())?;
        git::update_index(ledger, log::FILE, &sha)?;
        git::commit(ledger, &write_message(&event))?;
    }
    if let Err(error) = git::push_ff(ledger, branch) {
        if let Some(before) = before {
            git::reset_mixed(ledger, &before)?;
        }
        return Err(error);
    }
    report.pushed = Some(Range {
        from: committed + 1,
        to: working_seq(ledger)?,
    });
    Ok(())
}

/// Refuse to push writer-less rows (n-64be, ac-1e24): every uncommitted row
/// carries `by`. Only rows this sync would newly place on the remote count;
/// committed rows stay for the rewrite refusal (n-f4cd). The first push to
/// an empty remote is exempt: it establishes the ledger. Held rows stay
/// local, so the next sync says the same.
pub(super) fn refuse_writerless(ledger: &Path, upstream: &str) -> Result<()> {
    if git::rev(ledger, upstream).is_none() {
        return Ok(());
    }
    let committed = seq_at(ledger, "HEAD")?;
    let missing: Vec<String> = log::read(ledger)?
        .0
        .into_iter()
        .filter(|event| event.seq > committed && event.by.is_none())
        .map(|event| event.seq.to_string())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    Err(Error::invalid(format!(
        "writes {} have no writer (by): an older gy wrote them while this copy was not syncing. They stay in this copy, and nothing is pushed or pulled until they are gone. Keep this copy local (remove `remote` from gy.toml), or note what they say (gy show), remove the copy, run gy remote join, and write them again",
        missing.join(", ")
    )))
}

/// The first push owed when an empty remote has a committed copy; this also
/// recovers a first sync whose push failed before landing.
pub(super) fn first_push(
    ledger: &Path,
    remote: &str,
    branch: &str,
    report: &mut Sync,
) -> Result<()> {
    let heads = git::remote_refs(ledger, remote)?.1;
    if heads.iter().any(|head| head == branch) {
        return Ok(());
    }
    if !heads.is_empty() {
        return Err(Error::invalid(format!("the remote has no {branch} branch")));
    }
    let seq = seq_at(ledger, "HEAD")?;
    git::push(ledger, branch)?;
    git::fetch(ledger)?;
    report.pushed = Some(Range { from: 1, to: seq });
    report.reuploaded = state::synced_before(ledger);
    report.guard = Some(shape::GUIDANCE.to_string());
    Ok(())
}

/// The commit message of one write: its why, the actor, the human when the
/// copy records one, and the sequence.
fn write_message(event: &log::Event) -> String {
    let mut message = format!("{}\n\nactor: {}\n", event.why, event.actor);
    if let Some(by) = &event.by {
        message.push_str(&format!("by: {by}\n"));
        if let Some(mail) = &event.by_mail {
            message.push_str(&format!("by_mail: {mail}\n"));
        }
    }
    message.push_str(&format!("Gy-Seq: {}\n", event.seq));
    message
}
