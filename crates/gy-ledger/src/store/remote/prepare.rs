//! Making the missing copy: clone the remote's ledger branch, or initialize an
//! empty remote from the local ledger (n-6f47). The dedicated check comes
//! before any history comparison.
use super::super::{Error, Result, log};
use super::sync::{diverged, seq_at, working_seq};
use super::{git, shape};
use std::path::Path;

/// Make the missing copy: clone the remote's ledger branch (returning its
/// sequence), or initialize an empty remote from the local ledger.
pub(super) fn prepare(ledger: &Path, remote: &str) -> Result<Option<u64>> {
    let Some(branch) = one_branch(ledger, remote)? else {
        initialize(ledger, remote)?;
        return Ok(None);
    };
    if ledger.join(log::FILE).is_file() {
        let files = probe(ledger, remote, &branch)?;
        if !shape::dedicated(&files) {
            return Err(not_ledger(&files.join(", ")));
        }
        return Err(diverged());
    }
    clone(ledger, remote, &branch)?;
    Ok(Some(seq_at(ledger, "HEAD")?))
}

/// Reconcile the copy's `remote` marker with `gy.toml` before a command runs
/// (n-8a52, ac-fd1b): refuse a different remote, remove the marker and return
/// its URL once when gy.toml no longer names one, and clone when the copy is
/// missing and the remote has a ledger. Nothing is returned when they agree.
pub fn reconcile(ledger: &Path, remote: Option<&str>) -> Result<Option<String>> {
    let marker = std::fs::read_to_string(ledger.join("remote"))
        .ok()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty());
    match (marker, remote) {
        (Some(marked), Some(remote)) if marked != remote => Err(Error::invalid(format!(
            "gy.toml names a different remote than this copy ({marked}); remove the copy to start over"
        ))),
        (Some(marked), None) => {
            std::fs::remove_file(ledger.join("remote"))?;
            Ok(Some(marked))
        }
        (Some(_), Some(_)) => Ok(None),
        (None, Some(remote)) => {
            acquire(ledger, remote)?;
            Ok(None)
        }
        (None, None) => Ok(None),
    }
}

/// Clone the remote's ledger when this machine has no copy yet. A local
/// ledger, or an empty remote, is left to `gy sync` and to open (n-8a52).
fn acquire(ledger: &Path, remote: &str) -> Result<()> {
    if git::is_repo(ledger) || ledger.join(log::FILE).is_file() {
        return Ok(());
    }
    std::fs::create_dir_all(ledger)?;
    let Some(branch) = one_branch(ledger, remote)? else {
        return Ok(());
    };
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
