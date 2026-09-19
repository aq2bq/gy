//! Sending the copy's writes to the remote (n-6a8d): the push side of `gy
//! sync`, moved out of `sync.rs` with no change in behaviour.
use super::super::{Error, Result, log};
use super::report::{Range, Sync};
use super::sync::{locked, seq_at, working_seq};
use super::{git, guard, shape, state};
use std::path::Path;

/// Send the copy's commits: the remote must be behind, then push (committing
/// each uncommitted write first).
pub(super) fn push(
    ledger: &Path,
    branch: &str,
    upstream: &str,
    committed: u64,
    report: &mut Sync,
) -> Result<()> {
    guard::check_push(ledger, branch, upstream)?;
    let remote_seq = seq_at(ledger, upstream)?;
    if committed > remote_seq {
        git::push_ff(ledger, branch)?;
        report.pushed = Some(Range {
            from: remote_seq + 1,
            to: committed,
        });
    } else {
        locked(ledger, || {
            push_writes(ledger, branch, seq_at(ledger, "HEAD")?, report)
        })?;
    }
    report.seq = working_seq(ledger)?;
    Ok(())
}

/// Commit each uncommitted write as its own commit and push the branch. A
/// failed push rolls the commits back, so HEAD stays behind upstream and the
/// next sync tries again (n-ecbf 2B).
pub(super) fn push_writes(
    ledger: &Path,
    branch: &str,
    committed: u64,
    report: &mut Sync,
) -> Result<()> {
    let before = git::rev(ledger, "HEAD");
    let base = git::show(ledger, "HEAD", log::FILE).unwrap_or_default();
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
