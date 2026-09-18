//! A shared copy records who wrote (n-8a52, ac-11b2): git's configured human
//! lands in the event line and the commit message. Local test config only.
use gy_ledger::{CriterionAdd, FileStore, FormatVersion, Operation, Repository, format, log, sync};
use std::path::Path;
use std::process::Command;

/// No ambient git identity: each test sets exactly what it wants.
fn hermetic() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        for (key, value) in [
            ("GIT_AUTHOR_NAME", "piko"),
            ("GIT_AUTHOR_EMAIL", "piko@example.com"),
            ("GIT_COMMITTER_NAME", "piko"),
            ("GIT_COMMITTER_EMAIL", "piko@example.com"),
        ] {
            std::env::set_var(key, value);
        }
    });
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

/// A copy directory with a format file and, optionally, the `remote` marker.
fn copy(where_: &Path, marker: Option<&str>) -> std::path::PathBuf {
    let dir = where_.join("copy");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    git(&dir, &["init", "-q"]);
    if let Some(url) = marker {
        std::fs::write(dir.join("remote"), url).unwrap();
    }
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
fn a_marked_copy_records_by_and_by_mail() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let dir = copy(temp.path(), Some("file:///example/ledger.git"));
    git(&dir, &["config", "user.name", "piko"]);
    git(&dir, &["config", "user.email", "piko@example.com"]);

    write(&dir).unwrap();
    let events = log::read(&dir).unwrap().0;
    let event = events.first().expect("one write");
    assert_eq!(event.by.as_deref(), Some("piko"));
    assert_eq!(event.by_mail.as_deref(), Some("piko@example.com"));
}

#[test]
fn an_unmarked_copy_records_no_human() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let dir = copy(temp.path(), None);

    write(&dir).unwrap();
    let events = log::read(&dir).unwrap().0;
    let event = events.first().expect("one write");
    assert!(event.by.is_none() && event.by_mail.is_none());
    let text = std::fs::read_to_string(dir.join(log::FILE)).unwrap();
    assert!(!text.contains("\"by\""), "{text}");
}

#[test]
fn a_marked_copy_without_a_name_refuses_the_write() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let dir = copy(temp.path(), Some("file:///example/ledger.git"));
    git(&dir, &["config", "user.email", "piko@example.com"]);

    let error = write(&dir).unwrap_err();
    assert!(
        error.message.contains("user.name is not set"),
        "{}",
        error.message
    );
    assert!(log::read(&dir).unwrap().0.is_empty());
}

#[test]
fn sync_names_the_human_in_each_commit() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote) = (temp.path().join("ledger"), temp.path().join("remote.git"));
    std::fs::create_dir_all(&ledger).unwrap();
    format::write(&ledger, FormatVersion::CURRENT).unwrap();
    write(&ledger).unwrap();
    bare(&remote);
    sync(&ledger, &url(&remote)).unwrap();

    git(&ledger, &["config", "user.name", "piko"]);
    git(&ledger, &["config", "user.email", "piko@example.com"]);
    write(&ledger).unwrap();
    sync(&ledger, &url(&remote)).unwrap();

    let message = git(&remote, &["log", "-1", "--format=%B", "main"]);
    assert!(message.contains("by: piko"), "{message}");
    assert!(message.contains("by_mail: piko@example.com"), "{message}");
}
