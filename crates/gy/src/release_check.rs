//! The new-release notice (n-670a, r-23a9): a machine-local cache under
//! `base_dir()/gy`, refreshed by a detached `cargo info gy` at most once a
//! day. Reading commands only read the cache; a finished check's output file
//! is picked up – and an overrunning cargo reaped – by the next command, so
//! no command ever waits and no new setting, subcommand, or variable exists.
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

/// Seconds between checks, and seconds a check may run (n-670a).
const STALE_AFTER: u64 = 24 * 60 * 60;
const GIVE_UP_AFTER: u64 = 30;

const CACHE_FILE: &str = "release-check.json";
const PENDING_FILE: &str = "release-check.pending";
const OUT_FILE: &str = "release-check.out";

/// The cache: when last checked, and the newest release seen, if any.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Cache {
    checked_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest: Option<String>,
}

/// A check in flight: whose output to read, and when it started.
#[derive(Debug, Serialize, Deserialize)]
struct Pending {
    pid: u32,
    started_at: u64,
}

/// Read the cache, maybe refresh it in the background, and say on stderr
/// when a newer release is known. Never fails the command.
pub fn poll() {
    let dir = gy_ledger::location::base_dir().join("gy");
    if stale(&dir) {
        let running = settle(&dir);
        if !running && stale(&dir) {
            spawn(&dir);
        }
    }
    if let Some(latest) = read_cache(&dir).latest {
        let own = env!("CARGO_PKG_VERSION");
        if newer(own, &latest) {
            eprintln!(
                "gy {latest} is available (you have {own}); update with: cargo install gy --locked"
            );
        }
    }
}

/// The cache is missing, unreadable, or older than a day.
fn stale(dir: &Path) -> bool {
    match read_cache_opt(dir) {
        None => true,
        Some(cache) => cache.checked_at.saturating_add(STALE_AFTER) <= now(),
    }
}

/// Adopt a finished check, or give up waiting for an overrunning one. True
/// while a young check is still in flight. Nothing in flight writes nothing.
/// Never kills: the pid may already belong to someone else (n-670a 追記 2),
/// so a readable answer wins over the pid, and an overdue check is left
/// alone to end by itself.
fn settle(dir: &Path) -> bool {
    let Some(pending) = read_pending(dir) else {
        if dir.join(OUT_FILE).is_file() {
            adopt_out(dir);
        }
        return false;
    };
    if parse_version(&read_out(dir)).is_some() {
        adopt_out(dir);
        let _ = std::fs::remove_file(dir.join(PENDING_FILE));
        return false;
    }
    if process_alive(pending.pid) {
        if pending.started_at.saturating_add(GIVE_UP_AFTER) > now() {
            return true;
        }
        remove_flight(dir);
        stamp_without_version(dir);
        return false;
    }
    adopt_out(dir);
    let _ = std::fs::remove_file(dir.join(PENDING_FILE));
    false
}

/// Read a finished check's output file into the cache, keeping the previous
/// release when the output does not name one.
fn read_out(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(OUT_FILE)).unwrap_or_default()
}

fn adopt_out(dir: &Path) {
    let latest = parse_version(&read_out(dir)).or_else(|| read_cache(dir).latest);
    write_cache(
        dir,
        &Cache {
            checked_at: now(),
            latest,
        },
    );
    let _ = std::fs::remove_file(dir.join(OUT_FILE));
}

/// A failed check still counts for a day: the time moves, the release stays.
fn stamp_without_version(dir: &Path) {
    let latest = read_cache(dir).latest;
    write_cache(
        dir,
        &Cache {
            checked_at: now(),
            latest,
        },
    );
}

/// Start `cargo info gy` in the data directory, detached, its output to a
/// file the next command reads. A missing cargo counts as a failed check.
fn spawn(dir: &Path) {
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let out = match std::fs::File::create(dir.join(OUT_FILE)) {
        Ok(out) => out,
        Err(_) => return,
    };
    let mut command = Command::new("cargo");
    command
        .arg("info")
        .arg("gy")
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let Ok(child) = command.spawn() else {
        let _ = std::fs::remove_file(dir.join(OUT_FILE));
        stamp_without_version(dir);
        return;
    };
    let pending = Pending {
        pid: child.id(),
        started_at: now(),
    };
    let _ =
        serde_json::to_string(&pending).map(|text| std::fs::write(dir.join(PENDING_FILE), text));
}

/// The first `version: X.Y.Z` line, exactly three numbers. A `(from …)` line
/// or anything else never matches (n-670a).
fn parse_version(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("version:")?.trim();
        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
        {
            Some(rest.to_string())
        } else {
            None
        }
    })
}

/// True when `latest` is strictly newer than `own`, numerically.
fn newer(own: &str, latest: &str) -> bool {
    match (triple(own), triple(latest)) {
        (Some(own), Some(latest)) => latest > own,
        _ => false,
    }
}

fn triple(text: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let numbers: Vec<u64> = parts.iter().filter_map(|part| part.parse().ok()).collect();
    if numbers.len() != 3 {
        return None;
    }
    Some((numbers[0], numbers[1], numbers[2]))
}

fn read_cache(dir: &Path) -> Cache {
    read_cache_opt(dir).unwrap_or_default()
}

fn read_cache_opt(dir: &Path) -> Option<Cache> {
    std::fs::read_to_string(dir.join(CACHE_FILE))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
}

fn read_pending(dir: &Path) -> Option<Pending> {
    std::fs::read_to_string(dir.join(PENDING_FILE))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
}

fn remove_flight(dir: &Path) {
    let _ = std::fs::remove_file(dir.join(PENDING_FILE));
    let _ = std::fs::remove_file(dir.join(OUT_FILE));
}

fn write_cache(dir: &Path, cache: &Cache) {
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    if let Ok(text) = serde_json::to_string(cache) {
        let _ = std::fs::write(dir.join(CACHE_FILE), text);
    }
}

/// Best effort: `kill -0` says the pid is still alive (n-ecbf).
fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
