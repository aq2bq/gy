//! `gy sync` (n-6f47, d-39f6, d-1e50): prepare the copy, fetch, pull, and
//! push each local write as one commit.
use super::super::{Error, Result, SNAPSHOT_FILE, file::FileStore, log};
use super::{git, prepare};
use fs2::FileExt;
use serde::Serialize;
use std::fmt;
use std::path::Path;

/// A span of write sequences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Range {
    pub from: u64,
    pub to: u64,
}

/// The writes pulled, and the distinct actors they carry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pulled {
    pub from: u64,
    pub to: u64,
    pub writers: Vec<String>,
}

/// What one sync did. `seq` is the copy's sequence afterwards; it is not in
/// the JSON, whose absent sides are null.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Sync {
    pub pulled: Option<Pulled>,
    pub pushed: Option<Range>,
    #[serde(skip)]
    pub seq: u64,
}
impl fmt::Display for Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pulled) = &self.pulled {
            write!(
                f,
                "pulled: seq {} → {} ({} writes",
                pulled.from,
                pulled.to,
                pulled.to - pulled.from
            )?;
            if !pulled.writers.is_empty() {
                write!(f, " by {}", pulled.writers.join(", "))?;
            }
            writeln!(f, ")")?;
        }
        if let Some(pushed) = &self.pushed {
            writeln!(
                f,
                "pushed: {} writes (seq {} → {})",
                pushed.to - pushed.from + 1,
                pushed.from,
                pushed.to
            )?;
        }
        if self.pulled.is_none() && self.pushed.is_none() {
            writeln!(f, "up to date: seq {}", self.seq)?;
        }
        Ok(())
    }
}

/// Sync the ledger's copy with `remote`: clone or initialize it when missing,
/// fetch, fast-forward or push. The lock is held from fetch until the last
/// commit and push finish.
pub fn sync(ledger: &Path, remote: &str) -> Result<Sync> {
    std::fs::create_dir_all(ledger)?;
    let cloned = if git::is_repo(ledger) {
        None
    } else {
        prepare::prepare(ledger, remote)?
    };
    let _lock = Lock::take(ledger)?;
    let mut report = Sync::default();
    if let Some(seq) = cloned {
        report.pulled = Some(Pulled {
            from: 0,
            to: seq,
            writers: writers(ledger, 0, seq)?,
        });
        report.seq = seq;
    }
    git::fetch(ledger)?;
    let branch = git::head_branch(ledger)?;
    let upstream = format!("origin/{branch}");
    first_push(ledger, remote, &branch, &upstream, &mut report)?;
    reconcile(ledger, branch.as_str(), &upstream, &mut report)?;
    Ok(report)
}

/// Pull or push, or refuse a divergence. A copy with local commits already
/// made (a push that failed) pushes them; one with uncommitted lines commits
/// and pushes them one write each.
fn reconcile(ledger: &Path, branch: &str, upstream: &str, report: &mut Sync) -> Result<()> {
    let committed = seq_at(ledger, "HEAD")?;
    let remote_seq = seq_at(
        ledger,
        &git::rev(ledger, upstream)
            .ok_or_else(|| Error::invalid(format!("the remote has no {branch} branch")))?,
    )?;
    let working = working_seq(ledger)?;
    if remote_seq > committed && working > committed {
        return Err(diverged());
    }
    if remote_seq > committed {
        git::reset_hard(ledger, upstream)?;
        rebuild_snapshot(ledger)?;
        report.pulled = Some(Pulled {
            from: committed,
            to: remote_seq,
            writers: writers(ledger, committed, remote_seq)?,
        });
        report.seq = remote_seq;
    } else if committed > remote_seq {
        git::push_ff(ledger, branch)?;
        report.pushed = Some(Range {
            from: remote_seq + 1,
            to: committed,
        });
        report.seq = committed;
    } else if working > committed {
        push_writes(ledger, branch, committed, report)?;
        report.seq = working;
    } else {
        report.seq = committed;
    }
    Ok(())
}

/// Commit each uncommitted write as its own commit and push the branch.
fn push_writes(ledger: &Path, branch: &str, committed: u64, report: &mut Sync) -> Result<()> {
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
        git::commit(ledger, &write_message(&event.why, &event.actor, event.seq))?;
    }
    git::push_ff(ledger, branch)?;
    report.pushed = Some(Range {
        from: committed + 1,
        to: working_seq(ledger)?,
    });
    Ok(())
}

/// The first push owed when an empty remote has a committed copy; this also
/// recovers a first sync whose push failed before landing.
fn first_push(
    ledger: &Path,
    remote: &str,
    branch: &str,
    upstream: &str,
    report: &mut Sync,
) -> Result<()> {
    if git::rev(ledger, upstream).is_some() {
        return Ok(());
    }
    if !git::remote_refs(ledger, remote)?.1.is_empty() {
        return Err(Error::invalid(format!("the remote has no {branch} branch")));
    }
    let seq = seq_at(ledger, "HEAD")?;
    git::push(ledger, branch)?;
    git::fetch(ledger)?;
    report.pushed = Some(Range { from: 1, to: seq });
    Ok(())
}

/// The commit message of one write: its why, the actor, and the sequence.
fn write_message(why: &str, actor: &str, seq: u64) -> String {
    format!("{why}\n\nactor: {actor}\nGy-Seq: {seq}\n")
}

/// The distinct actors of the writes in `(from, to]`, in first-seen order.
fn writers(ledger: &Path, from: u64, to: u64) -> Result<Vec<String>> {
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
fn rebuild_snapshot(ledger: &Path) -> Result<()> {
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
