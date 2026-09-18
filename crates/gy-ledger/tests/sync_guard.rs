//! The remote history's shape (n-f4cd, ac-af33): gy refuses a history changed
//! outside gy and prints how to recover. Local bare remotes only.
use gy_ledger::{CriterionAdd, FileStore, FormatVersion, Operation, Repository, format, log, sync};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The commit identity the child git inherits.
fn ident() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
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

fn set_user(dir: &Path) {
    git(dir, &["config", "user.name", "piko"]);
    git(dir, &["config", "user.email", "piko@example.com"]);
}

/// A local ledger with one write; returns its last sequence.
fn seed(dir: &Path) -> u64 {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    write(dir).unwrap();
    last_seq(dir)
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

fn last_seq(dir: &Path) -> u64 {
    log::read(dir).unwrap().0.last().unwrap().seq
}

/// The first sync's copy, plus a second copy cloned from the same remote.
fn pair(temp: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let first = temp.join("first");
    let remote = temp.join("remote.git");
    seed(&first);
    bare(&remote);
    sync(&first, &url(&remote)).unwrap();
    let second = temp.join("second");
    sync(&second, &url(&remote)).unwrap();
    (first, second, remote)
}

#[test]
fn refuses_a_commit_without_a_trailer() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second, remote) = pair(temp.path());
    set_user(&second);
    write(&second).unwrap();
    git(&second, &["add", "-A"]);
    git(&second, &["commit", "-q", "-m", "hand edit"]);
    git(&second, &["push", "-q", "origin", "HEAD:main"]);

    let error = sync(&first, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("no Gy-Seq trailer"),
        "{}",
        error.message
    );
    assert!(
        error.message.contains("force-with-lease="),
        "{}",
        error.message
    );
}

#[test]
fn refuses_a_rewritten_events_line() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second, remote) = pair(temp.path());
    let next = last_seq(&first) + 1;
    let text = std::fs::read_to_string(second.join(log::FILE)).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    std::fs::write(
        second.join(log::FILE),
        format!("{}\n", lines[..lines.len() - 1].join("\n")),
    )
    .unwrap();
    git(&second, &["add", "-A"]);
    git(
        &second,
        &["commit", "-q", "-m", &format!("edit\n\nGy-Seq: {next}")],
    );
    git(&second, &["push", "-q", "origin", "HEAD:main"]);

    let error = sync(&first, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("rewrites events.jsonl"),
        "{}",
        error.message
    );
}

#[test]
fn refuses_a_commit_touching_another_file() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second, remote) = pair(temp.path());
    let next = last_seq(&first) + 1;
    std::fs::write(second.join("notes.txt"), "hello").unwrap();
    git(&second, &["add", "-A"]);
    git(
        &second,
        &["commit", "-q", "-m", &format!("notes\n\nGy-Seq: {next}")],
    );
    git(&second, &["push", "-q", "origin", "HEAD:main"]);

    let error = sync(&first, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("touches notes.txt"),
        "{}",
        error.message
    );
}

#[test]
fn refuses_a_rewritten_remote_history() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second, remote) = pair(temp.path());
    git(&second, &["commit", "--amend", "-m", "hand rewritten"]);
    git(&second, &["push", "-q", "--force", "origin", "HEAD:main"]);

    let error = sync(&first, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("not a descendant"),
        "{}",
        error.message
    );
}

#[test]
fn a_changed_remote_with_unpushed_writes_reads_as_a_guard_refusal() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second, remote) = pair(temp.path());
    // The local copy has a write that is not pushed yet.
    set_user(&first);
    write(&first).unwrap();
    // The remote gains a commit gy did not make.
    git(&second, &["commit", "--allow-empty", "-m", "hand edit"]);
    git(&second, &["push", "-q", "origin", "HEAD:main"]);

    let error = sync(&first, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("no Gy-Seq trailer"),
        "{}",
        error.message
    );
    assert!(!error.message.contains("diverged"), "{}", error.message);
}

#[test]
fn the_first_push_prints_the_guidance_once() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let remote = temp.path().join("remote.git");
    seed(&first);
    bare(&remote);

    let pushed = sync(&first, &url(&remote)).unwrap();
    assert!(pushed.guard.is_some());
    assert!(format!("{pushed}").contains("require linear history"));
    let json = serde_json::to_value(&pushed).unwrap();
    assert!(
        json["guard"]
            .as_str()
            .unwrap()
            .contains("block force pushes")
    );

    let again = sync(&first, &url(&remote)).unwrap();
    assert!(again.guard.is_none());
    assert!(!format!("{again}").contains("require linear history"));
}

#[test]
fn a_correct_history_is_pulled() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (first, second, remote) = pair(temp.path());
    set_user(&second);
    write(&second).unwrap();
    let next = last_seq(&second);
    let pushed = sync(&second, &url(&remote)).unwrap();
    assert_eq!(pushed.pushed.unwrap().to, next);

    let pulled = sync(&first, &url(&remote)).unwrap();
    assert_eq!(pulled.pulled.unwrap().to, next);
    assert_eq!(last_seq(&first), next);
}
