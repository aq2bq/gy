//! The new-release notice (n-670a, r-23a9): a fake `cargo` on `PATH`, never
//! the real crates.io. The cache lives under `XDG_DATA_HOME`, outside the
//! work tree (d-aa21).
mod common;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const OWN: &str = env!("CARGO_PKG_VERSION");

fn cache_file(fx: &common::Fixture) -> PathBuf {
    fx.data.join("gy").join("release-check.json")
}

fn write_cache(fx: &common::Fixture, checked_at: u64, latest: Option<&str>) {
    let mut text = format!("{{\"checked_at\":{checked_at}");
    if let Some(latest) = latest {
        text.push_str(&format!(",\"latest\":\"{latest}\""));
    }
    text.push('}');
    let path = cache_file(fx);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A fake `cargo` answering `info gy` with `body`, recording calls in
/// `marker` when given.
fn fake_cargo(dir: &Path, body: &str, marker: Option<&Path>) {
    let mut script = format!("#!/bin/sh\ncat <<'EOF'\n{body}\nEOF\n");
    if let Some(marker) = marker {
        script.push_str(&format!("touch {}\n", marker.display()));
    }
    let path = dir.join("cargo");
    std::fs::write(&path, script).unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
}

fn run_with_path(fx: &common::Fixture, args: &[&str], path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&fx.root)
        .env("XDG_DATA_HOME", &fx.data)
        .env("GY_ACTOR", "piko")
        .env("PATH", path)
        .args(args)
        .output()
        .unwrap()
}

fn bin_path(temp: &tempfile::TempDir) -> (PathBuf, String) {
    let dir = temp.path().join("bin");
    std::fs::create_dir_all(&dir).unwrap();
    let path = format!("{}:/usr/bin:/bin", dir.display());
    (dir, path)
}

/// A pid known dead: spawned and reaped, so reuse cannot confuse the test.
fn dead_pid() -> u32 {
    let mut child = Command::new("sh").arg("-c").arg("exit 0").spawn().unwrap();
    let pid = child.id();
    child.wait().unwrap();
    pid
}

/// A newer release reads from the cache alone, on stderr only (i).
#[test]
fn newer_release_is_announced_from_cache() {
    let temp = tempfile::tempdir().unwrap();
    let (bindir, path) = bin_path(&temp);
    let marker = temp.path().join("called");
    fake_cargo(&bindir, "version: 9.9.9\n", Some(&marker));
    let fx = common::fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);
    write_cache(&fx, now_secs(), Some("9.9.9"));

    let out = run_with_path(&fx, &["handover"], &path);
    assert!(out.status.success(), "{}", common::stderr(&out));
    let err = common::stderr(&out);
    assert!(err.contains("9.9.9"), "{err}");
    assert!(err.contains("cargo install gy --locked"), "{err}");
    assert!(!common::stdout(&out).contains("9.9.9"));
    // Fresh cache: no query spawned.
    assert!(!marker.exists());
}

/// The same release stays silent (ii).
#[test]
fn same_release_stays_silent() {
    let temp = tempfile::tempdir().unwrap();
    let (bindir, path) = bin_path(&temp);
    fake_cargo(&bindir, "version: 9.9.9\n", None);
    let fx = common::fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);
    write_cache(&fx, now_secs(), Some(OWN));

    let out = run_with_path(&fx, &["next"], &path);
    assert!(out.status.success(), "{}", common::stderr(&out));
    assert!(!common::stderr(&out).contains("available"));
    assert!(!common::stdout(&out).contains("available"));
}

/// No cargo: silent, and only the time is written (iii).
#[test]
fn missing_cargo_writes_only_the_time() {
    let temp = tempfile::tempdir().unwrap();
    let emptydir = temp.path().join("empty");
    std::fs::create_dir_all(&emptydir).unwrap();
    let fx = common::fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);

    let out = run_with_path(&fx, &["handover"], &emptydir.display().to_string());
    assert!(out.status.success(), "{}", common::stderr(&out));
    assert!(!common::stderr(&out).contains("available"));
    let text = std::fs::read_to_string(cache_file(&fx)).unwrap();
    assert!(text.contains("checked_at"), "{text}");
    assert!(!text.contains("latest"), "{text}");
}

/// An unreadable answer: silent, version kept (iv).
#[test]
fn unreadable_answer_keeps_silent() {
    let temp = tempfile::tempdir().unwrap();
    let (bindir, path) = bin_path(&temp);
    fake_cargo(&bindir, "garbage{{{}}}\n", None);
    let fx = common::fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);
    // A completed stale query: dead pid, old start, garbage output.
    let dir = fx.data.join("gy");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("release-check.pending"),
        format!("{{\"pid\":{},\"started_at\":0}}", dead_pid()),
    )
    .unwrap();
    std::fs::write(dir.join("release-check.out"), "garbage{{{}}}\n").unwrap();

    let out = run_with_path(&fx, &["handover"], &path);
    assert!(out.status.success(), "{}", common::stderr(&out));
    assert!(!common::stderr(&out).contains("available"));
    let text = std::fs::read_to_string(cache_file(&fx)).unwrap();
    assert!(!text.contains("latest"), "{text}");
}

/// A `(from …)` line alone is not a release (v).
#[test]
fn workspace_answer_is_not_a_release() {
    let temp = tempfile::tempdir().unwrap();
    let (bindir, path) = bin_path(&temp);
    fake_cargo(&bindir, "version: 9.9.9 (from ./crates/gy)\n", None);
    let fx = common::fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);
    let dir = fx.data.join("gy");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("release-check.pending"),
        format!("{{\"pid\":{},\"started_at\":0}}", dead_pid()),
    )
    .unwrap();
    std::fs::write(
        dir.join("release-check.out"),
        "version: 9.9.9 (from ./crates/gy)\n",
    )
    .unwrap();

    let out = run_with_path(&fx, &["handover"], &path);
    assert!(out.status.success(), "{}", common::stderr(&out));
    assert!(!common::stderr(&out).contains("available"));
    let text = std::fs::read_to_string(cache_file(&fx)).unwrap();
    assert!(!text.contains("latest"), "{text}");
}

/// An overrunning check is never killed (追記 2): a live unrelated process
/// whose pid sits in an old pending file survives the command.
#[test]
fn overrunning_check_is_left_alone() {
    let temp = tempfile::tempdir().unwrap();
    let (_bindir, path) = bin_path(&temp);
    let fx = common::fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);
    let mut sleeper = Command::new("sleep").arg("30").spawn().unwrap();
    let dir = fx.data.join("gy");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("release-check.pending"),
        format!("{{\"pid\":{},\"started_at\":0}}", sleeper.id()),
    )
    .unwrap();

    let out = run_with_path(&fx, &["handover"], &path);
    assert!(out.status.success(), "{}", common::stderr(&out));
    assert!(!common::stderr(&out).contains("available"));
    // The stranger is still running, not even a zombie; only the time was
    // written. (`kill -0` answers a zombie too, so ask the child.)
    assert!(
        sleeper.try_wait().unwrap().is_none(),
        "an unrelated process was killed"
    );
    let text = std::fs::read_to_string(dir.join("release-check.json")).unwrap();
    assert!(text.contains("checked_at"), "{text}");
    assert!(!dir.join("release-check.pending").exists());
    let _ = sleeper.kill();
}

/// A spawned query lands in the cache, and the next command announces (i end
/// to end). A fresh cache spawns nothing (vi).
#[test]
fn spawned_query_lands_and_next_command_announces() {
    let temp = tempfile::tempdir().unwrap();
    let (bindir, path) = bin_path(&temp);
    fake_cargo(&bindir, "version: 9.9.9\n", None);
    let fx = common::fixture();
    fx.seed(&[common::criterion("0001", "an ac")]);

    // Each run either adopts the finished query or leaves a young one alone;
    // the adopting run announces from the fresh cache in the same pass.
    let started = std::time::Instant::now();
    loop {
        let out = run_with_path(&fx, &["handover"], &path);
        assert!(out.status.success(), "{}", common::stderr(&out));
        if let Ok(text) = std::fs::read_to_string(cache_file(&fx)) {
            if text.contains("9.9.9") {
                assert!(
                    common::stderr(&out).contains("9.9.9"),
                    "{}",
                    common::stderr(&out)
                );
                break;
            }
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "no cache landed"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}
