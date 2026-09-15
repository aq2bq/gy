//! Finding the repository root and opening its ledger for reading. Reads never
//! create a ledger; the first write does (N-63).
use gy_ledger::{Error, FileStore, Repository, Result, format};
use std::path::{Path, PathBuf};

/// A placeholder actor for reads. Only a write names the real `GY_ACTOR`.
const READ_ACTOR: &str = "gy5-read";

/// The directory that holds `gy.toml`, starting at `start` (or the current
/// directory) and walking up.
pub fn root(start: Option<&Path>) -> Result<PathBuf> {
    let mut dir = match start {
        Some(dir) => dir
            .canonicalize()
            .map_err(|_| Error::invalid(format!("no directory {}", dir.display())))?,
        None => std::env::current_dir()?,
    };
    loop {
        if dir.join("gy.toml").is_file() {
            return Ok(dir);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => return Err(Error::invalid("no gy.toml found; pass -C <dir>")),
        }
    }
}

/// Open the ledger at `ledger` for reading. A missing ledger is an error and
/// is never created here.
pub fn open(ledger: &Path) -> Result<Repository<FileStore>> {
    if !ledger.join(format::FILE).is_file() {
        return Err(Error::invalid(format!("no ledger at {}", ledger.display())));
    }
    let store = FileStore::open_with(ledger, |_| Some(READ_ACTOR.to_string()))?;
    Ok(Repository::new(store))
}
