//! `gy share <URL>`: check a remote, then the first upload (n-57c5, ac-efd7).
//! The gy.toml write and the order of the three live in the CLI; this module
//! knows git and the words. Nothing here changes a remote: the push check is a
//! dry run in a throwaway repository.
use super::super::{Error, Gate, Result, log};
use super::sync::sync_with;
use super::{git, shape};
use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// The remote after the checks passed: the `checked:` line to print and the
/// branch the first upload will write.
#[derive(Debug)]
pub struct Checked {
    pub line: String,
    pub branch: String,
}

/// What `gy share` prints, in order, one line per prefix (ac-efd7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Share {
    pub lines: Vec<String>,
}

impl Share {
    /// The lines of a share that passed the checks: `checked:`, `wrote:`, the
    /// upload, the protection, and the invitation.
    pub fn shared(url: &str, checked: String, mut uploaded: Vec<String>) -> Self {
        let mut lines = vec![checked, format!("wrote: remote = \"{url}\" in gy.toml")];
        lines.append(&mut uploaded);
        lines.extend(
            shape::GUIDANCE
                .lines()
                .map(|line| format!("protect: {line}")),
        );
        lines.push(
            "invite: ask a member to get write access, then run `gy join` in their checkout"
                .to_string(),
        );
        Self { lines }
    }

    /// The one line of a share that finds the project already shared (exit 0).
    pub fn already(url: &str) -> Self {
        Self {
            lines: vec![format!("checked: already shared with {url}")],
        }
    }
}

impl fmt::Display for Share {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

/// Check the remote before anything is written (ac-efd7 b): it is reachable,
/// empty or a gy ledger only, and this machine can push to it. An error names
/// the cause and the way out; nothing is changed.
pub fn check(root: &Path, url: &str) -> Result<Checked> {
    let (head, heads) = git::remote_refs(root, url).map_err(unreachable)?;
    if heads.len() > 1 {
        return Err(not_ledger("it has more than one branch"));
    }
    let branch = match heads.first() {
        Some(only) => head
            .filter(|head| head == only)
            .unwrap_or_else(|| only.clone()),
        None => head.unwrap_or_else(|| "main".to_string()),
    };
    if !heads.is_empty() {
        let files = probe(url, &branch)?;
        if !shape::dedicated(&files) {
            return Err(not_ledger(&files.join(", ")));
        }
    }
    push_check(url, &branch)?;
    let line = if heads.is_empty() {
        format!("checked: {url} is empty")
    } else {
        format!("checked: {url} holds a gy ledger (branch {branch})")
    };
    Ok(Checked { line, branch })
}

/// Upload the local ledger as the first copy (ac-efd7 d). A machine with no
/// ledger yet keeps only the gy.toml write; the first write will start it. The
/// gate is the caller's, so the first upload is judged by this build's rule
/// (n-f921): the store owns the order, the ops layer owns the rule.
pub fn upload(ledger: &Path, url: &str, branch: &str, gate: Arc<dyn Gate>) -> Result<Vec<String>> {
    if !ledger.join(log::FILE).is_file() {
        return Ok(vec![
            "uploaded: no ledger yet; the first write will start it, then gy sync uploads it"
                .to_string(),
        ]);
    }
    let report = sync_with(ledger, url, gate).map_err(|error| {
        Error::invalid(format!(
            "the upload failed: {}\nthe remote stays in gy.toml; fix the access and run gy sync",
            error.message
        ))
    })?;
    let line = match report.pushed {
        Some(range) => format!(
            "uploaded: seq {} → {} as one commit to {branch}",
            range.from, range.to
        ),
        None => format!("uploaded: the ledger is on {url} ({branch})"),
    };
    Ok(vec![line])
}

/// The tracked files of the remote's branch, cloned into a throwaway directory
/// so a non-ledger is judged before anything is written.
fn probe(url: &str, branch: &str) -> Result<Vec<String>> {
    let scratch = Scratch::new()?;
    let tree = scratch.0.join("tree");
    std::fs::create_dir_all(&tree)?;
    git::quiet(&tree, &["clone", "-q", "--branch", branch, url, "."])?;
    git::ls_tree(&tree, "HEAD")
}

/// Prove the push permission with a dry run from a throwaway repository.
/// `git::quiet` keeps git's chatter off the reader's stderr (n-57c5).
fn push_check(url: &str, branch: &str) -> Result<()> {
    let scratch = Scratch::new()?;
    let repo = scratch.0.join("repo");
    std::fs::create_dir_all(&repo)?;
    git::init(&repo, branch)?;
    probe_commit(&repo)?;
    git::quiet(
        &repo,
        &[
            "push",
            "--dry-run",
            url,
            &format!("HEAD:refs/heads/{branch}"),
        ],
    )
    .map_err(|_| {
        Error::invalid(format!(
            "the remote refused the push (no write access?): {url}\nask the repository owner for write access"
        ))
    })
}

/// One throwaway empty commit, with its own identity, so the dry run has a ref
/// to push. The scratch directory is not a ledger copy, so no marker is needed.
fn probe_commit(dir: &Path) -> Result<()> {
    git::run(
        dir,
        &[
            "-c",
            "user.name=gy",
            "-c",
            "user.email=gy@invalid",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "gy share check",
        ],
    )
    .map(|_| ())
}

/// A throwaway directory under the system temp, removed when it drops.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("gy-share-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        Ok(Self(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The message a remote that is not a ledger reads.
fn not_ledger(what: &str) -> Error {
    Error::invalid(format!(
        "the remote is not a gy ledger ({what}); point gy at a ledger-only repository"
    ))
}

/// The message an unreachable remote reads, with the way to check it.
fn unreachable(error: Error) -> Error {
    Error::invalid(format!(
        "cannot reach the remote: {}\ncheck the URL and your git credentials (git ls-remote <URL>), and that this machine is online",
        error.message
    ))
}
