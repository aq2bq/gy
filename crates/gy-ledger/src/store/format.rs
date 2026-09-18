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

/// Write the version atomically (a temporary file, then a rename). The
/// temporary name carries the process id, so two writers creating a ledger at
/// once do not clobber each other's rename (n-fe59).
pub fn write(dir: &Path, version: FormatVersion) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let temporary = dir.join(format!(".format.{}.tmp", std::process::id()));
    std::fs::write(&temporary, format!("{}\n", version.0))?;
    std::fs::rename(&temporary, dir.join(FILE))?;
    Ok(())
}

/// Migrate the ledger in place, one step at a time up to `to`. Every step
/// keeps a copy of the file it changes before writing the new version (D-77):
/// 1 to 2 copies `format` and `events.jsonl` to `*.1.bak` (n-ff2b); 2 to 3
/// copies `format` to `*.2.bak` and drops the derived snapshot, which the next
/// open rebuilds from the log (n-b6b6). A jump this build does not know is an
/// error.
pub fn migrate(dir: &Path, from: FormatVersion, to: FormatVersion) -> Result<()> {
    if from == to {
        return Ok(());
    }
    let next = FormatVersion(from.0 + 1);
    match from.0 {
        1 => {
            backup(dir, FILE, 1)?;
            backup(dir, super::log::FILE, 1)?;
        }
        2 => {
            backup(dir, FILE, 2)?;
            drop_snapshot(dir)?;
        }
        _ => {
            return Err(Error::invalid(format!(
                "cannot migrate format {} to {}",
                from.0, to.0
            )));
        }
    }
    write(dir, next)?;
    migrate(dir, next, to)
}

/// Keep a copy of a file as `<name>.<from>.bak` before a version migration.
fn backup(dir: &Path, name: &str, from: u32) -> Result<()> {
    let source = dir.join(name);
    if source.is_file() {
        std::fs::copy(&source, dir.join(format!("{name}.{from}.bak")))?;
    }
    Ok(())
}

/// Drop the derived snapshot so the next open rebuilds it from the log, where
/// every old `created` is normalized (n-b6b6).
fn drop_snapshot(dir: &Path) -> Result<()> {
    let path = dir.join(super::snapshot::FILE);
    if path.is_file() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
