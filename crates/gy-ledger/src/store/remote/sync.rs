//! `gy sync` (n-6f47, d-39f6, d-1e50): prepare the copy, fetch, and
//! fast-forward. In this step only sync pushes; pushing each local write is
//! the next step (n-8a52).
use super::super::{Error, Result, SNAPSHOT_FILE, file::FileStore, log};
use super::{git, shape};
use fs2::FileExt;
use serde::Serialize;
use std::fmt;
use std::path::Path;

/// What one sync did: the sequences pulled, and the sequences pushed when the
/// first sync initialized an empty remote.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Sync {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulled: Option<(u64, u64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pushed: Option<(u64, u64)>,
}
impl fmt::Display for Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((from, to)) = self.pulled {
            writeln!(f, "pulled: seq {from} → {to}")?;
        }
        if let Some((from, to)) = self.pushed {
            writeln!(f, "pushed: seq {from} → {to}")?;
        }
        Ok(())
    }
}

/// Sync the ledger's copy with `remote`: clone or initialize it when missing,
/// fetch, and fast-forward a copy that has no write of its own. The lock is
/// held from fetch until the replace and snapshot rebuild finish.
pub fn sync(ledger: &Path, remote: &str) -> Result<Sync> {
    std::fs::create_dir_all(ledger)?;
    if !git::is_repo(ledger) {
        prepare(ledger, remote)?;
    }
    let _lock = Lock::take(ledger)?;
    let mut report = Sync::default();
    git::fetch(ledger)?;
    let branch = git::head_branch(ledger)?;
    let upstream = format!("origin/{branch}");
    first_push(ledger, remote, &branch, &upstream, &mut report)?;
    let origin_sha = git::rev(ledger, &upstream)
        .ok_or_else(|| Error::invalid(format!("the remote has no {branch} branch")))?;
    pull(ledger, &origin_sha, &mut report)?;
    Ok(report)
}

/// Fast-forward a copy that has no write of its own. A copy with unpushed
/// writes, with or without the remote ahead, is left to the push step (A2).
fn pull(ledger: &Path, origin_sha: &str, report: &mut Sync) -> Result<()> {
    let committed = seq_at(ledger, "HEAD")?;
    let remote_seq = seq_at(ledger, origin_sha)?;
    let working = working_seq(ledger)?;
    if remote_seq > committed && working == committed {
        git::reset_hard(ledger, origin_sha)?;
        rebuild_snapshot(ledger)?;
        report.pulled = Some((committed, remote_seq));
    } else if remote_seq != committed || working > committed {
        return Err(Error::invalid(
            "this copy has writes that are not pushed yet; pushing is the next step",
        ));
    }
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
    report.pushed = Some((1, seq));
    Ok(())
}

/// Make the missing copy: clone the remote's ledger branch, or initialize an
/// empty remote from the local ledger. A remote that is not a ledger is
/// refused before any history is compared.
fn prepare(ledger: &Path, remote: &str) -> Result<()> {
    let Some(branch) = one_branch(ledger, remote)? else {
        return initialize(ledger, remote);
    };
    if ledger.join(log::FILE).is_file() {
        let files = probe(ledger, remote, &branch)?;
        if !shape::dedicated(&files) {
            return Err(not_ledger(&files.join(", ")));
        }
        return Err(Error::invalid(
            "this copy and the remote have diverged; decide which is canonical and remove one copy, then sync again",
        ));
    }
    clone(ledger, remote, &branch)
}

/// The remote's one ledger branch, or `None` when it has no branch yet. Two
/// or more branches is not a ledger.
fn one_branch(dir: &Path, url: &str) -> Result<Option<String>> {
    let (advertised, heads) = git::remote_refs(dir, url)?;
    if heads.is_empty() {
        return Ok(None);
    }
    if let Some(head) = advertised.filter(|head| heads.contains(head)) {
        return Ok(Some(head));
    }
    if heads.len() == 1 {
        return Ok(heads.into_iter().next());
    }
    Err(not_ledger("it has more than one branch"))
}

/// Initialize an empty remote from the local ledger (the first sync).
fn initialize(ledger: &Path, remote: &str) -> Result<()> {
    if !ledger.join(log::FILE).is_file() {
        return Err(Error::invalid(
            "nothing to sync: the remote and this copy are both empty",
        ));
    }
    let branch = git::remote_refs(ledger, remote)?
        .0
        .unwrap_or_else(|| "main".into());
    git::init(ledger, &branch)?;
    git::add_remote(ledger, remote)?;
    std::fs::write(ledger.join(shape::README_FILE), shape::README)?;
    std::fs::write(ledger.join(shape::GITIGNORE_FILE), shape::GITIGNORE)?;
    std::fs::write(ledger.join("remote"), remote)?;
    let seq = working_seq(ledger)?;
    git::add_all(ledger)?;
    git::commit(ledger, &shape::initial_message(seq))?;
    Ok(())
}

/// Clone the ledger branch into the copy and refuse a non-ledger, leaving no
/// half-clone behind.
fn clone(ledger: &Path, remote: &str, branch: &str) -> Result<()> {
    git::clone_branch(ledger, remote, branch)?;
    let files = git::ls_tree(ledger, "HEAD")?;
    if !shape::dedicated(&files) {
        std::fs::remove_dir_all(ledger)?;
        std::fs::create_dir_all(ledger)?;
        return Err(not_ledger(&files.join(", ")));
    }
    std::fs::write(ledger.join("remote"), remote)?;
    Ok(())
}

/// The tracked files of the remote's branch, cloned into a throwaway directory
/// so an occupied ledger directory can still be judged first.
fn probe(ledger: &Path, remote: &str, branch: &str) -> Result<Vec<String>> {
    let probe = ledger.join(".probe");
    let _ = std::fs::remove_dir_all(&probe);
    std::fs::create_dir_all(&probe)?;
    let files =
        git::clone_branch(&probe, remote, branch).and_then(|()| git::ls_tree(&probe, "HEAD"));
    let _ = std::fs::remove_dir_all(&probe);
    files
}

/// The message a remote that is not a ledger reads.
fn not_ledger(what: &str) -> Error {
    Error::invalid(format!(
        "the remote is not a gy ledger ({what}); point gy.toml at a ledger-only repository"
    ))
}

/// The last sequence in the working log.
fn working_seq(ledger: &Path) -> Result<u64> {
    Ok(log::read(ledger)?.0.last().map_or(0, |event| event.seq))
}

/// The last sequence in `events.jsonl` at a revision (0 when absent).
fn seq_at(dir: &Path, revision: &str) -> Result<u64> {
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
