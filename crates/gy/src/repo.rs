//! Finding the repository root and opening its ledger for reading. Reads never
//! create a ledger; the first write does (N-63).
use gy_ledger::{Actor, Error, FileStore, FormatVersion, Repository, Result, format};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A placeholder actor for reads. Only a write names the real `GY_ACTOR`.
const READ_ACTOR: &str = "gy-read";

/// `start`, or the current directory, as a real path.
fn here(start: Option<&Path>) -> Result<PathBuf> {
    match start {
        Some(dir) => dir
            .canonicalize()
            .map_err(|_| Error::invalid(format!("no directory {}", dir.display()))),
        None => Ok(std::env::current_dir()?),
    }
}

/// The directory `gy init` writes into: `-C` if given, else the current
/// directory, canonicalized so the report names a real path (n-29b3).
pub fn init_dir(start: Option<&Path>) -> Result<PathBuf> {
    here(start)
}

/// The nearest ancestor of `start` (or the current directory) that holds
/// `gy.toml`, or `None` when there is none (n-29b3).
pub fn find_root(start: Option<&Path>) -> Result<Option<PathBuf>> {
    let mut dir = here(start)?;
    loop {
        if dir.join("gy.toml").is_file() {
            return Ok(Some(dir));
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => return Ok(None),
        }
    }
}

/// The directory that holds `gy.toml`, starting at `start` (or the current
/// directory) and walking up.
pub fn root(start: Option<&Path>) -> Result<PathBuf> {
    find_root(start)?.ok_or_else(|| {
        Error::invalid(
            "no gy.toml found; pass -C <dir> to name the repository,\n\
             or run gy init <scope> to start one",
        )
    })
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

/// Start a detached `gy remote sync` after a write to a shared copy, unless one is
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
        .arg("remote")
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

/// Say once, on stderr, which refused writes this copy still carries (n-ecbf
/// 2B2). Standard output is never touched.
pub fn announce_rejected(ledger: &Path) {
    for notice in gy_ledger::rejected_notices(ledger) {
        eprintln!("{notice}");
    }
}

/// Reach the remote before handover: run `gy remote sync` and give it five seconds,
/// then keep the copy as it stands (n-ecbf 2B2).
pub fn refresh(root: &Path, ledger: &Path) -> Result<()> {
    if !ledger.join("remote").is_file() {
        return Ok(());
    }
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg("-C")
        .arg(root)
        .arg("remote")
        .arg("sync")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            gy_ledger::record_timeout_after(ledger, 5);
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// What one `gy serve` sync tick did (n-94bb).
pub enum Tick {
    /// A sync ran and moved the copy; the first line to log.
    Out(String),
    /// A sync ran but failed; the last error's first line.
    Failed(String),
    /// A sync ran with nothing to say (`up to date`).
    Silent,
    /// Another sync holds the copy; this tick does nothing.
    Skipped,
}

/// One sync for `gy serve`: a child `gy remote sync` with a ten second cap, its
/// stdout read for the first line (n-94bb). A background sync already running
/// skips the tick.
pub fn sync_tick(root: &Path, ledger: &Path) -> Tick {
    if read_pid(&ledger.join("sync.pid")).is_some_and(process_alive) {
        return Tick::Skipped;
    }
    let Some(mut child) = spawn_sync(root) else {
        return Tick::Failed("could not start gy remote sync".to_string());
    };
    let _ = std::fs::write(ledger.join("sync.pid"), child.id().to_string());
    if !wait_capped(&mut child, ledger) {
        return Tick::Failed(last_error(ledger));
    }
    let Ok(output) = child.wait_with_output() else {
        return Tick::Failed(last_error(ledger));
    };
    if !output.status.success() {
        return Tick::Failed(last_error(ledger));
    }
    let first = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    if first.starts_with("up to date") {
        Tick::Silent
    } else {
        Tick::Out(first)
    }
}

/// The child `gy remote sync` serve waits for, with its output on a pipe.
fn spawn_sync(root: &Path) -> Option<std::process::Child> {
    let mut command = Command::new(std::env::current_exe().ok()?);
    command
        .arg("-C")
        .arg(root)
        .arg("remote")
        .arg("sync")
        .env("GY_SYNC_BACKGROUND", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command.spawn().ok()
}

/// Wait for the child, killing it and recording a give-up at ten seconds.
fn wait_capped(child: &mut std::process::Child, ledger: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().ok().flatten().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            gy_ledger::record_timeout_after(ledger, 10);
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    true
}

/// The first line of the copy's `sync.state` last error, when there is one.
fn last_error(ledger: &Path) -> String {
    std::fs::read_to_string(ledger.join("sync.state"))
        .ok()
        .and_then(|text| serde_json::from_str::<gy_ledger::SyncStatus>(&text).ok())
        .and_then(|state| state.last_error)
        .map(|error| error.lines().next().unwrap_or_default().to_string())
        .unwrap_or_else(|| "sync failed".to_string())
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
