//! The `git` child process, in one place (n-6f47, d-39f6). Nothing above knows
//! git exists; it never prompts, and a missing git is its own error.
use super::super::{Error, Result};
use super::origin;
use std::path::Path;
use std::process::{Command, Output};

/// Run git in `dir`, returning stdout. A failed command is an error carrying
/// git's own message.
pub fn run(dir: &Path, args: &[&str]) -> Result<String> {
    let output = spawn(dir, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::invalid(format!(
            "git {}: {}",
            args.join(" "),
            stderr.trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run git and return stdout, or `None` on a non-zero exit.
fn run_maybe(dir: &Path, args: &[&str]) -> Option<String> {
    run(dir, args).ok()
}

/// The configured `origin` URL, or `None` when the copy has no origin.
pub(super) fn origin(dir: &Path) -> Option<String> {
    run_maybe(dir, &["remote", "get-url", "origin"]).map(|url| url.trim().to_string())
}

/// Spawn the child process. This is the only place git is executed.
fn spawn(dir: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::invalid("git is required for a remote")
            } else {
                Error::invalid(format!("git: {error}"))
            }
        })
}

/// Whether `dir` is its own git work tree (a prepared copy), not a directory
/// inside another work tree. The judgment is `rev-parse --show-toplevel`
/// naming `dir`; a parent's toplevel does not count (ac-ad40). Every plumbing
/// path below is relative to `dir`, which this equality makes the tree's root.
pub fn is_repo(dir: &Path) -> bool {
    let Some(toplevel) = run_maybe(dir, &["rev-parse", "--show-toplevel"]) else {
        return false;
    };
    match (
        std::fs::canonicalize(dir),
        std::fs::canonicalize(toplevel.trim()),
    ) {
        (Ok(dir), Ok(top)) => dir == top,
        _ => false,
    }
}

/// Create an empty work tree on `branch`.
pub fn init(dir: &Path, branch: &str) -> Result<()> {
    run(dir, &["init", "-q", "-b", branch]).map(|_| ())
}

/// Clone `url`'s `branch` into `dir`, which must exist and be empty. The
/// branch is named, so a remote whose HEAD is unborn still checks out.
pub fn clone_branch(dir: &Path, url: &str, branch: &str) -> Result<()> {
    run(dir, &["clone", "-q", "--branch", branch, url, "."]).map(|_| ())
}

/// Point the work tree at `url` as `origin`.
pub fn add_remote(dir: &Path, url: &str) -> Result<()> {
    run(dir, &["remote", "add", "origin", url]).map(|_| ())
}

/// Fetch the remote's branches.
pub fn fetch(dir: &Path) -> Result<()> {
    origin::ensure(dir)?;
    run(dir, &["fetch", "-q", "origin"]).map(|_| ())
}

/// The remote's advertised default branch and its branch names, from one git
/// call. Empty heads means the remote has no branch yet. A local bare remote
/// with an unborn HEAD advertises nothing.
pub fn remote_refs(dir: &Path, url: &str) -> Result<(Option<String>, Vec<String>)> {
    let out = run(dir, &["ls-remote", "--symref", url])?;
    let mut head = None;
    let mut heads = Vec::new();
    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("ref: refs/heads/") {
            head = rest.split_whitespace().next().map(str::to_string);
        } else if let Some(name) = line
            .split_whitespace()
            .last()
            .and_then(|name| name.strip_prefix("refs/heads/"))
        {
            heads.push(name.to_string());
        }
    }
    Ok((head, heads))
}

/// The configured human: git's `user.name` and `user.email`, in one call. An
/// unset pair is `(None, None)`; a missing git is an error.
pub fn user(dir: &Path) -> Result<(Option<String>, Option<String>)> {
    let output = spawn(dir, &["config", "--get-regexp", r"^user\.(name|email)$"])?;
    if !output.status.success() {
        return Ok((None, None));
    }
    let mut name = None;
    let mut mail = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        match line.split_once(' ') {
            Some(("user.name", value)) => name = Some(value.to_string()),
            Some(("user.email", value)) => mail = Some(value.to_string()),
            _ => {}
        }
    }
    Ok((name, mail))
}

/// The current branch name.
pub fn head_branch(dir: &Path) -> Result<String> {
    run(dir, &["symbolic-ref", "--short", "HEAD"]).map(|text| text.trim().to_string())
}

/// The commit a revision names, or `None` when it does not exist.
pub fn rev(dir: &Path, revision: &str) -> Option<String> {
    run_maybe(dir, &["rev-parse", "--verify", "--quiet", revision])
        .map(|text| text.trim().to_string())
        .filter(|sha| !sha.is_empty())
}

/// Move the work tree and index to `revision` (the fast-forward).
pub fn reset_hard(dir: &Path, revision: &str) -> Result<()> {
    origin::ensure(dir)?;
    run(dir, &["reset", "-q", "--hard", revision]).map(|_| ())
}

/// Move HEAD to `revision` and keep the work tree, so a failed push can leave
/// the commits out while the writes stay (n-ecbf 2B).
pub fn reset_mixed(dir: &Path, revision: &str) -> Result<()> {
    origin::ensure(dir)?;
    run(dir, &["reset", "-q", revision]).map(|_| ())
}

/// Stage every change under the work tree.
pub fn add_all(dir: &Path) -> Result<()> {
    origin::ensure(dir)?;
    run(dir, &["add", "-A"]).map(|_| ())
}

/// Commit the index with `message`.
pub fn commit(dir: &Path, message: &str) -> Result<()> {
    origin::ensure(dir)?;
    run(dir, &["commit", "-q", "-m", message]).map(|_| ())
}

/// Push the current `branch` to origin and set its upstream.
pub fn push(dir: &Path, branch: &str) -> Result<()> {
    origin::ensure(dir)?;
    run(dir, &["push", "-q", "-u", "origin", branch]).map(|_| ())
}

/// Push `branch` to origin (the upstream is already set). A refusal names the
/// likely cause: a missing write permission.
pub fn push_ff(dir: &Path, branch: &str) -> Result<()> {
    origin::ensure(dir)?;
    let output = spawn(dir, &["push", "origin", branch])?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = stderr.trim();
    if [
        "Permission",
        "permission",
        "denied",
        "403",
        "Authentication",
        "not authorized",
        "unpack failed",
        "unable to create",
    ]
    .iter()
    .any(|word| text.contains(word))
    {
        return Err(Error::invalid(format!(
            "the remote refused the push (no write access?): {text}\nask the repository owner for write access, or keep reading this copy without pushing"
        )));
    }
    Err(Error::invalid(format!(
        "git push origin {branch}: {text}\ncheck the remote URL and your credentials, then run gy sync again"
    )))
}

/// Write `bytes` into the object database and return the blob's sha.
pub fn hash_object(dir: &Path, bytes: &[u8]) -> Result<String> {
    use std::io::Write;
    origin::ensure(dir)?;
    let mut child = Command::new("git")
        .args(["hash-object", "-w", "--stdin"])
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| Error::invalid(format!("git: {error}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| Error::invalid("git: no stdin"))?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(Error::invalid(format!(
            "git hash-object: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Point the index's `path` at an already-written blob, so one commit can hold
/// exactly the lines written so far.
pub fn update_index(dir: &Path, path: &str, sha: &str) -> Result<()> {
    origin::ensure(dir)?;
    run(
        dir,
        &["update-index", "--add", "--cacheinfo", "100644", sha, path],
    )
    .map(|_| ())
}

/// The tracked files at `revision`.
pub fn ls_tree(dir: &Path, revision: &str) -> Result<Vec<String>> {
    run(dir, &["ls-tree", "-r", "--name-only", revision])
        .map(|text| text.lines().map(str::to_string).collect())
}

/// The text of `path` at `revision`, or `None` when the file is absent.
pub fn show(dir: &Path, revision: &str, path: &str) -> Option<String> {
    run_maybe(dir, &["show", &format!("{revision}:{path}")])
}

/// Whether `ancestor` is an ancestor of `descendant`.
pub fn ancestor(dir: &Path, ancestor: &str, descendant: &str) -> bool {
    run_maybe(dir, &["merge-base", "--is-ancestor", ancestor, descendant]).is_some()
}

/// The commits in `range`, oldest first.
pub fn rev_list(dir: &Path, range: &str) -> Result<Vec<String>> {
    run(dir, &["rev-list", "--reverse", range])
        .map(|text| text.lines().map(str::to_string).collect())
}

/// A commit's message.
pub fn message(dir: &Path, revision: &str) -> Result<String> {
    run(dir, &["log", "-1", "--format=%B", revision])
}

/// A commit's author name.
pub fn author(dir: &Path, revision: &str) -> Result<String> {
    run(dir, &["log", "-1", "--format=%an", revision]).map(|text| text.trim().to_string())
}

/// The files a commit changed.
pub fn files_changed(dir: &Path, revision: &str) -> Result<Vec<String>> {
    run(
        dir,
        &["diff-tree", "--no-commit-id", "--name-only", "-r", revision],
    )
    .map(|text| text.lines().map(str::to_string).collect())
}

/// How many lines the diff from `from` to `to` deletes in `path`.
pub fn deletions(dir: &Path, from: &str, to: &str, path: &str) -> Result<u64> {
    let out = run(dir, &["diff", "--numstat", from, to, "--", path])?;
    Ok(out
        .lines()
        .next()
        .and_then(|line| line.split('\t').nth(1))
        .and_then(|deleted| deleted.parse().ok())
        .unwrap_or(0))
}
