//! A shared copy with no name to write under (n-8a52, d-a4f6). The author
//! environment is process-wide, so the cases where a name is present live in
//! `sync_copy.rs` and this file keeps the machine nameless: what git derives
//! from the account is not a name anyone chose, and gy refuses rather than
//! record it.
use gy_ledger::{CriterionAdd, FileStore, FormatVersion, Operation, Repository, format, log};
use std::path::Path;
use std::process::Command;

/// No name anywhere: no author environment, and an empty config.
fn nameless() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
        for key in [
            "GIT_AUTHOR_NAME",
            "GIT_AUTHOR_EMAIL",
            "GIT_COMMITTER_NAME",
            "GIT_COMMITTER_EMAIL",
        ] {
            std::env::remove_var(key);
        }
    });
}

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A copy directory carrying the remote marker.
fn copy(where_: &Path) -> std::path::PathBuf {
    let dir = where_.join("copy");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    git(&dir, &["init", "-q"]);
    std::fs::write(dir.join("remote"), "file:///example/ledger.git").unwrap();
    dir
}

fn write(dir: &Path) -> gy_ledger::Result<gy_ledger::Outcome<gy_ledger::NodeId>> {
    let mut repo = Repository::new(FileStore::open_with(dir, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "an ac".into(),
        body: None,
    }
    .run(&mut repo)
}

#[test]
fn a_marked_copy_without_a_name_refuses_the_write() {
    nameless();
    let temp = tempfile::tempdir().unwrap();
    let dir = copy(temp.path());
    git(&dir, &["config", "user.email", "piko@example.com"]);

    let error = write(&dir).unwrap_err();
    // Both ways to give a name, because either one would have done.
    assert!(
        error.message.contains("git config user.name") && error.message.contains("GIT_AUTHOR_NAME"),
        "{}",
        error.message
    );
    assert!(log::read(&dir).unwrap().0.is_empty());
}

#[test]
fn the_configured_name_alone_is_enough() {
    nameless();
    let temp = tempfile::tempdir().unwrap();
    let dir = copy(temp.path());
    git(&dir, &["config", "user.name", "alice"]);
    git(&dir, &["config", "user.email", "alice@example.com"]);

    write(&dir).unwrap();
    let events = log::read(&dir).unwrap().0;
    let event = events.first().expect("one write");
    assert_eq!(event.by.as_deref(), Some("alice"));
    assert_eq!(event.by_mail.as_deref(), Some("alice@example.com"));
}
