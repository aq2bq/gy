//! `gy remote sync` (n-6f47, d-39f6, d-1e50): prepare the copy, fetch, pull, and
//! push each local write as one commit.
use super::super::{Error, Gate, Result, SNAPSHOT_FILE, file::FileStore, log};
use super::push::{first_push, push, push_writes, raise_format, raise_target, refuse_writerless};
use super::report::{Pulled, Sync};
use super::{git, guard, origin, prepare, rebase, recovery, rules, state};
use fs2::FileExt;
use std::path::Path;
use std::sync::Arc;

/// Sync the ledger's copy with `remote`: prepare the copy, fetch, then change
/// it under the lock only when a change is due (n-ecbf). The state file is
/// written either way. `gate` judges each write a rebase places (n-557f).
#[doc(hidden)]
pub fn sync_with(ledger: &Path, remote: &str, gate: Arc<dyn Gate>) -> Result<Sync> {
    let _exclusive = state::SyncLock::take(ledger)?;
    let outcome = sync_inner(ledger, remote, &*gate);
    state::clear_step(ledger);
    state::record_state(ledger, &outcome);
    outcome
}

fn sync_inner(ledger: &Path, remote: &str, gate: &dyn Gate) -> Result<Sync> {
    std::fs::create_dir_all(ledger)?;
    if git::is_repo(ledger) {
        if let Some(message) = recovery::damaged(ledger)? {
            return Err(Error::invalid(message));
        }
    }
    let cloned = if git::is_repo(ledger) {
        None
    } else {
        state::set_step(ledger, "clone");
        prepare::prepare(ledger, remote)?
    };
    let mut report = Sync::default();
    if let Some(seq) = cloned {
        report.pulled = Some(Pulled {
            from: 0,
            to: seq,
            writers: writers(ledger, 0, seq)?,
        });
        report.seq = seq;
    }
    // Communication is outside the lock: a write is not blocked by a fetch.
    fetch(ledger)?;
    let branch = git::head_branch(ledger)?;
    let upstream = format!("origin/{branch}");
    refuse_writerless(ledger, &upstream)?;
    let remote_format = rules::check_remote_format(ledger, &upstream)?;
    let raise = raise_target(rules::local_format(ledger), remote_format);
    state::set_step(ledger, "push");
    first_push(ledger, remote, &branch, &mut report)?;
    reconcile(ledger, branch.as_str(), &upstream, &mut report, gate, raise)?;
    Ok(report)
}

/// Fetch the remote. The ownership refusal stays bare; only a real fetch
/// failure names the URL and the credentials (n-9f9d).
fn fetch(ledger: &Path) -> Result<()> {
    state::set_step(ledger, "fetch");
    origin::ensure(ledger)?;
    git::fetch(ledger).map_err(|error| {
        rules::advice(
            error,
            "check the remote URL and your git credentials (git remote -v, then git fetch), and that this machine is online",
        )
    })
}

/// Take the copy's exclusive lock only around a change, so a fetch or a push
/// does not hold it (n-ecbf).
pub(super) fn locked(ledger: &Path, f: impl FnOnce() -> Result<()>) -> Result<()> {
    let _lock = Lock::take(ledger)?;
    f()
}

/// Pull or push, or refuse a divergence. A copy with local commits already
/// made (a push that failed) pushes them; one with uncommitted lines commits
/// and pushes them one write each.
fn reconcile(
    ledger: &Path,
    branch: &str,
    upstream: &str,
    report: &mut Sync,
    gate: &dyn Gate,
    raise: Option<u32>,
) -> Result<()> {
    state::set_step(ledger, "push");
    let head =
        git::rev(ledger, "HEAD").ok_or_else(|| Error::invalid("the copy has no commit yet"))?;
    let Some(remote_sha) = git::rev(ledger, upstream) else {
        return Err(Error::invalid(format!("the remote has no {branch} branch")));
    };
    let committed = seq_at(ledger, "HEAD")?;
    if remote_sha == head {
        report.seq = committed;
        if working_seq(ledger)? > committed {
            locked(ledger, || {
                push_writes(ledger, branch, seq_at(ledger, "HEAD")?, report, raise)
            })?;
            report.seq = working_seq(ledger)?;
        } else if let Some(target) = raise {
            locked(ledger, || raise_format(ledger, branch, target))?;
        }
    } else if git::ancestor(ledger, "HEAD", upstream) {
        pull(ledger, branch, upstream, committed, report, gate, raise)?;
    } else {
        push(ledger, branch, upstream, committed, report, raise)?;
    }
    Ok(())
}

/// Take in the remote's commits: they pass the guard, then replace the copy.
fn pull(
    ledger: &Path,
    branch: &str,
    upstream: &str,
    committed: u64,
    report: &mut Sync,
    gate: &dyn Gate,
    raise: Option<u32>,
) -> Result<()> {
    // Shape first: a changed remote must read as a guard refusal, not as a
    // divergence that tells the reader to delete a copy (n-f4cd).
    guard::check(ledger, branch, upstream)?;
    if working_seq(ledger)? > committed {
        // The copy has its own uncommitted writes and the remote moved: put
        // them back on top of the remote's lines (n-ecbf 2B).
        state::set_step(ledger, "rebase");
        return locked(ledger, || {
            rebase::rebase(ledger, branch, upstream, committed, report, gate, raise)
        });
    }
    state::set_step(ledger, "fetch");
    locked(ledger, || {
        git::reset_hard(ledger, upstream)?;
        rebuild_snapshot(ledger)?;
        // The reset put the remote's older format in the work tree; write this
        // copy's back and raise the remote after taking it in (n-96f8).
        match raise {
            Some(target) => raise_format(ledger, branch, target),
            None => Ok(()),
        }
    })?;
    let remote_seq = seq_at(ledger, upstream)?;
    report.pulled = Some(Pulled {
        from: committed,
        to: remote_seq,
        writers: writers(ledger, committed, remote_seq)?,
    });
    report.seq = remote_seq;
    Ok(())
}

/// The distinct actors of the writes in `(from, to]`, in first-seen order.
pub(super) fn writers(ledger: &Path, from: u64, to: u64) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for event in log::read(ledger)?.0 {
        if event.seq > from && event.seq <= to && !out.contains(&event.actor) {
            out.push(event.actor);
        }
    }
    Ok(out)
}

/// The message a divergence reads.
pub(super) fn diverged() -> Error {
    Error::invalid(
        "this copy and the remote have diverged; decide which is canonical and remove one copy, then sync again",
    )
}

/// The last sequence in the working log.
pub(super) fn working_seq(ledger: &Path) -> Result<u64> {
    Ok(log::read(ledger)?.0.last().map_or(0, |event| event.seq))
}

/// The last sequence in `events.jsonl` at a revision (0 when absent).
pub(super) fn seq_at(dir: &Path, revision: &str) -> Result<u64> {
    let Some(text) = git::show(dir, revision, log::FILE) else {
        return Ok(0);
    };
    let mut last = 0;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let event: log::Event = serde_json::from_str(line)
            .map_err(|_| Error::invalid("a committed event is not an event"))?;
        last = event.seq;
    }
    Ok(last)
}

/// Drop the derived snapshot and rebuild it from the fetched log, under the
/// same lock (n-6f47).
pub(super) fn rebuild_snapshot(ledger: &Path) -> Result<()> {
    let _ = std::fs::remove_file(ledger.join(SNAPSHOT_FILE));
    FileStore::open_with(ledger, |_| Some("gy-read".to_string()))?;
    Ok(())
}

/// The ledger's lock, held from fetch until the sync ends so a write cannot
/// append while the work tree is replaced (n-6f47). Dropping the file releases
/// it.
struct Lock {
    _file: std::fs::File,
}
impl Lock {
    fn take(dir: &Path) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join("lock"))?;
        file.lock_exclusive()?;
        Ok(Self { _file: file })
    }
}
