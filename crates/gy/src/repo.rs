//! Finding the repository root and opening its ledger for reading. Reads never
//! create a ledger; the first write does (N-63).
use gy_ledger::{Actor, Error, FileStore, FormatVersion, Repository, Result, format};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A placeholder actor for reads. Only a write names the real `GY_ACTOR`.
const READ_ACTOR: &str = "gy-read";

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
    announce_migration(&store);
    Ok(Repository::new(store))
}

/// Open the ledger for writing, creating it on first use. The actor comes from
/// `GY_ACTOR`; an unset actor is an error.
pub fn open_write(ledger: &Path) -> Result<Repository<FileStore>> {
    Actor::from_env()?;
    if !ledger.join(format::FILE).is_file() {
        std::fs::create_dir_all(ledger)?;
        format::write(ledger, FormatVersion::CURRENT)?;
    }
    let store = FileStore::open(ledger)?;
    announce_migration(&store);
    Ok(Repository::new(store))
}

/// Tell the reader, on stderr, when this open ran a format migration (n-f8a2).
/// Standard output, including `--json`, is never touched.
fn announce_migration(store: &FileStore) {
    let Some((from, to)) = store.migrated() else {
        return;
    };
    let version = env!("CARGO_PKG_VERSION");
    eprintln!(
        "gy {version} migrated this ledger from format {from} to {to}; what changed for you: CHANGELOG {version} Updating (https://github.com/aq2bq/gy/blob/main/CHANGELOG.md)"
    );
}

/// Start a detached `gy sync` after a write to a shared copy, unless one is
/// already running (n-ecbf). Its output goes to the copy's `sync.log`, so the
/// write stays as fast as before.
pub fn background_sync(root: &Path, ledger: &Path) -> Result<()> {
    if !ledger.join("remote").is_file() {
        return Ok(());
    }
    let pid_path = ledger.join("sync.pid");
    if read_pid(&pid_path).is_some_and(process_alive) {
        return Ok(());
    }
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger.join("sync.log"))?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg("-C")
        .arg(root)
        .arg("sync")
        .env("GY_SYNC_BACKGROUND", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let child = command.spawn()?;
    std::fs::write(&pid_path, child.id().to_string())?;
    Ok(())
}

fn read_pid(path: &Path) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Best effort: `kill -0` says the pid is still alive (n-ecbf).
fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
