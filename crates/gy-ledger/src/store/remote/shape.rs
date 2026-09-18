//! What gy puts in the ledger repository, and what makes a remote a ledger
//! (n-6f47, ac-fd1b). The history-shape check of ac-af33 is the next step.
use super::super::{format, log};

/// The files gy writes into a ledger repository. A dedicated remote holds only
/// these (or nothing).
pub const FILES: [&str; 4] = [log::FILE, format::FILE, README_FILE, GITIGNORE_FILE];

/// The README file name.
pub const README_FILE: &str = "README.md";
/// The .gitignore file name.
pub const GITIGNORE_FILE: &str = ".gitignore";

/// The bytes of the ledger repository's `.gitignore`: the derived and local
/// files never belong in the remote.
pub const GITIGNORE: &str = "snapshot.json\nlock\n*.bak\n.*.tmp\nremote\n";

/// The bytes of the ledger repository's `README.md`: minimal, so a reader
/// knows not to touch it (ac-af33).
pub const README: &str = "\
# gy ledger

This repository is written by gy. Do not edit, merge, open pull requests, or
force push. To change the ledger, use gy.
";

/// The one commit of the first sync: the actor, and the range of sequences it
/// carries (ac-af33, C accepts both this and one sequence).
pub fn initial_message(seq: u64) -> String {
    let range = if seq == 0 {
        "0".to_string()
    } else {
        format!("1-{seq}")
    };
    format!("gy sync: initial copy\n\nGy-Seq: {range}\n")
}

/// Whether a remote's tracked files are only what gy puts there (or nothing).
pub fn dedicated(files: &[String]) -> bool {
    files.iter().all(|file| FILES.contains(&file.as_str()))
}
