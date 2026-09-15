//! The `format` file: the ledger's format version (D-77). A ledger carries a
//! version from birth, and opening one this build does not support is an
//! error. A migration keeps a copy of the old form and runs as one
//! transaction; N-41 fills in the migration itself.
use super::{Error, FormatVersion, Result};
use std::path::Path;

/// The file name that holds the version.
pub const FILE: &str = "format";

/// Read the version. A missing file, a non-number, or a version newer than
/// this build supports is an error.
pub fn read(dir: &Path) -> Result<FormatVersion> {
    let path = dir.join(FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|_| Error::invalid(format!("no readable format file at {}", path.display())))?;
    let version = FormatVersion(
        text.trim()
            .parse()
            .map_err(|_| Error::invalid("the format file is not a version number"))?,
    );
    if !version.supported() {
        return Err(Error::invalid(format!(
            "format {} is newer than this build supports",
            version.0
        )));
    }
    Ok(version)
}

/// Write the version atomically (a temporary file, then a rename).
pub fn write(dir: &Path, version: FormatVersion) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let temporary = dir.join(".format.tmp");
    std::fs::write(&temporary, format!("{}\n", version.0))?;
    std::fs::rename(&temporary, dir.join(FILE))?;
    Ok(())
}

/// Migrate the ledger in place. A version bump keeps a copy of the old form
/// before writing the new version (D-77): format 1 to 2 copies `format` and
/// `events.jsonl` to `*.1.bak` (n-ff2b).
pub fn migrate(dir: &Path, from: FormatVersion, to: FormatVersion) -> Result<()> {
    if from == to {
        return Ok(());
    }
    if from.0 == 1 && to.0 == 2 {
        backup(dir, FILE)?;
        backup(dir, super::log::FILE)?;
        return write(dir, to);
    }
    Err(Error::invalid(format!(
        "cannot migrate format {} to {}",
        from.0, to.0
    )))
}

/// Keep a copy of a file as `<name>.1.bak` before the version 1 to 2 migration.
fn backup(dir: &Path, name: &str) -> Result<()> {
    let source = dir.join(name);
    if source.is_file() {
        std::fs::copy(&source, dir.join(format!("{name}.1.bak")))?;
    }
    Ok(())
}
