//! The ownership guard for a ledger copy (n-6d6c, ac-ad40): a git operation
//! must not land in a repository the copy does not own. `is_repo` already names
//! the copy itself; this makes its `origin` agree with the recorded remote
//! before a commit or a push.
use super::super::{Error, Result};
use super::git;
use std::path::Path;

/// The file a copy records its remote in, beside the log. It is gitignored, so
/// it never travels to the remote.
const REMOTE_FILE: &str = "remote";

/// Whether this copy was ever shared: it is its own git repository with an
/// `origin` (n-9f9d, d-dc39). The marker may be gone while gy.toml's remote
/// is away; writes in that window still record their human. A never-shared
/// ledger is not a git repository, so it stays local (n-8a52). Without a
/// `.git` entry there is nothing to ask: git would otherwise walk up and
/// report a parent repository's origin (n-6d6c).
pub fn shared_copy(dir: &Path) -> bool {
    if !dir.join(".git").exists() {
        return false;
    }
    git::is_repo(dir) && git::origin(dir).is_some()
}

/// Refuse an operation that would change a repository this copy does not own.
/// The copy's `remote` marker binds it to `gy.toml`, which gy's command layer
/// keeps equal before every command; `origin` must name that same URL, and a
/// missing marker refuses outright. Every git function that could change the
/// work tree or the index calls this first (ac-ad40).
pub(super) fn ensure(dir: &Path) -> Result<()> {
    let expected = std::fs::read_to_string(dir.join(REMOTE_FILE))
        .ok()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty());
    let Some(expected) = expected else {
        return Err(Error::invalid(
            "this copy has no remote marker; refusing to touch the repository",
        ));
    };
    let origin = git::origin(dir);
    if origin.as_deref() == Some(expected.as_str()) {
        return Ok(());
    }
    Err(Error::invalid(format!(
        "the ledger directory belongs to another repository ({}); refusing to touch it",
        origin.as_deref().unwrap_or("no origin")
    )))
}
