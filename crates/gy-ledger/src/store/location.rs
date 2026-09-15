//! Where the canonical ledger lives: outside the repository, under the XDG
//! data directory, keyed by the repository root (AC-44, D-82). The repository
//! keeps only `gy.toml`.
use std::path::{Path, PathBuf};

/// The base data directory: `XDG_DATA_HOME`, or `$HOME/.local/share`.
pub fn base_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME").filter(|dir| !dir.is_empty()) {
        return PathBuf::from(dir);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".local/share")
}

/// The stable key for a repository root: fnv1a of its canonical path.
pub fn key(root: &Path) -> String {
    let path = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    format!("{:08x}", super::fnv1a(&path.to_string_lossy()))
}

/// The ledger directory for a root under an explicit base (tests pass one).
pub fn dir_in(base: &Path, root: &Path) -> PathBuf {
    base.join("gy").join(key(root))
}

/// The ledger directory for a root under the default base.
pub fn ledger_dir(root: &Path) -> PathBuf {
    dir_in(&base_dir(), root)
}
