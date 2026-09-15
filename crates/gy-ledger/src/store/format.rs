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
/// and applies the whole migration in one transaction (D-77). N-41 fills this
/// in; only a same-version call is a no-op today.
pub fn migrate(from: FormatVersion, to: FormatVersion) -> Result<()> {
    if from == to {
        return Ok(());
    }
    Err(Error::invalid("ledger migration arrives with N-41"))
}
