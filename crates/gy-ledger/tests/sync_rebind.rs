//! Rebinding a copy whose marker is gone (n-9f9d, d-dc39): when gy.toml names
//! the remote again and the copy's origin still names that same URL, the
//! marker is written back and the sync continues. Anything else keeps
//! refusing, and the refusal carries no unrelated URL advice.
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, Operation, Repository, format, log, reconcile, sync,
};
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

fn git(cwd: &Path, args: &[&str]) {
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
}

fn url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

/// A bound copy: one write synced to the bare remote, so the copy has both
/// the `remote` marker and an `origin`.
fn bound(temp: &Path) -> (std::path::PathBuf, String) {
    let (ledger, remote) = (temp.join("ledger"), temp.join("remote.git"));
    std::fs::create_dir_all(&ledger).unwrap();
    format::write(&ledger, FormatVersion::CURRENT).unwrap();
    write(&ledger).unwrap();
    bare(&remote);
    let remote_url = url(&remote);
    sync(&ledger, &remote_url).unwrap();
    assert!(ledger.join("remote").is_file());
    (ledger, remote_url)
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

/// The marker returns when the same remote is named again (d-dc39), and the
/// writes made while it was gone still reach the remote.
#[test]
fn reconcile_rebinds_when_origin_matches() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote_url) = bound(temp.path());
    assert_eq!(
        reconcile(&ledger, None).unwrap().as_deref(),
        Some(remote_url.as_str())
    );
    assert!(!ledger.join("remote").exists());

    assert_eq!(reconcile(&ledger, Some(&remote_url)).unwrap(), None);
    assert_eq!(
        std::fs::read_to_string(ledger.join("remote")).unwrap(),
        remote_url
    );

    write(&ledger).unwrap();
    let report = sync(&ledger, &remote_url).unwrap();
    assert_eq!(report.seq, 2);
}

/// A different origin is not rebound: the ownership check holds (ac-ad40).
#[test]
fn reconcile_does_not_rebind_when_origin_differs() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote_url) = bound(temp.path());
    reconcile(&ledger, None).unwrap();
    git(
        &ledger,
        &["remote", "set-url", "origin", "file:///elsewhere.git"],
    );

    assert_eq!(reconcile(&ledger, Some(&remote_url)).unwrap(), None);
    assert!(!ledger.join("remote").exists());
}

/// No origin is not rebound either.
#[test]
fn reconcile_does_not_rebind_when_origin_is_missing() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote_url) = bound(temp.path());
    reconcile(&ledger, None).unwrap();
    git(&ledger, &["remote", "remove", "origin"]);

    assert_eq!(reconcile(&ledger, Some(&remote_url)).unwrap(), None);
    assert!(!ledger.join("remote").exists());
}

/// A markerless but shared copy still records its human (n-9f9d, r-0a90-5):
/// the copy has an origin even while the marker is away.
#[test]
fn markerless_but_shared_writes_carry_by() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, _) = bound(temp.path());
    reconcile(&ledger, None).unwrap();
    assert!(!ledger.join("remote").exists());

    write(&ledger).unwrap();
    let events = log::read(&ledger).unwrap().0;
    let event = events.last().expect("two writes");
    assert_eq!(event.by.as_deref(), Some("piko"));
    assert_eq!(event.by_mail.as_deref(), Some("piko@example.com"));
}

/// A never-shared local ledger reads no git config (n-8a52).
#[test]
fn local_writes_carry_no_by() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("ledger");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();

    write(&dir).unwrap();
    let events = log::read(&dir).unwrap().0;
    let event = events.first().expect("one write");
    assert!(event.by.is_none() && event.by_mail.is_none());
}

/// Writer-less rows are not pushed: the reason and the way out read
/// (n-64be, ac-1e24). Held rows stay local; the next sync says the same.
#[test]
fn sync_holds_writerless_rows_with_reason_and_way_out() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote_url) = bound(temp.path());
    reconcile(&ledger, None).unwrap();
    git(&ledger, &["remote", "remove", "origin"]);
    // Written while the copy looked local: no writer is recorded.
    write(&ledger).unwrap();
    assert!(log::read(&ledger).unwrap().0.last().unwrap().by.is_none());
    // The origin is back and the marker is rebound, yet the push refuses.
    git(&ledger, &["remote", "add", "origin", &remote_url]);
    assert_eq!(reconcile(&ledger, Some(&remote_url)).unwrap(), None);

    let error = sync(&ledger, &remote_url).unwrap_err();
    assert!(error.message.contains("writes 2"), "{}", error.message);
    assert!(error.message.contains("no writer"), "{}", error.message);
    assert!(
        error.message.contains("nothing is pushed or pulled"),
        "{}",
        error.message
    );
    assert!(
        error.message.contains("write them again"),
        "{}",
        error.message
    );
    let again = sync(&ledger, &remote_url).unwrap_err();
    assert_eq!(again.message, error.message);
}

/// A never-shared record inside a git repository stays local (n-6d6c): the
/// parent's origin must not mark it shared, and no git config is read.
#[test]
fn record_inside_a_foreign_repo_carries_no_by() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let parent = temp.path().join("parent");
    std::fs::create_dir_all(&parent).unwrap();
    git(&parent, &["init", "-q"]);
    git(
        &parent,
        &["remote", "add", "origin", "file:///example/dotfiles.git"],
    );
    git(&parent, &["config", "user.name", "trap-user"]);
    git(&parent, &["config", "user.email", "trap@example.com"]);

    let dir = parent.join("ledger");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    write(&dir).unwrap();

    let events = log::read(&dir).unwrap().0;
    let event = events.first().expect("one write");
    assert!(event.by.is_none() && event.by_mail.is_none());
    let text = std::fs::read_to_string(dir.join(log::FILE)).unwrap();
    assert!(!text.contains("trap-user"), "{text}");
    assert!(!text.contains("\"by\""), "{text}");
}

/// A missing marker refuses without the unrelated URL advice (ac-ab35).
#[test]
fn sync_without_a_marker_refuses_without_url_advice() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote_url) = bound(temp.path());
    reconcile(&ledger, None).unwrap();

    let error = sync(&ledger, &remote_url).unwrap_err();
    assert!(
        error.message.contains("no remote marker"),
        "{}",
        error.message
    );
    assert!(
        !error.message.contains("check the remote URL"),
        "{}",
        error.message
    );
}

/// A foreign origin refuses without the unrelated URL advice either.
#[test]
fn sync_with_a_foreign_origin_refuses_without_url_advice() {
    hermetic();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, remote_url) = bound(temp.path());
    git(
        &ledger,
        &["remote", "set-url", "origin", "file:///elsewhere.git"],
    );

    let error = sync(&ledger, &remote_url).unwrap_err();
    assert!(
        error.message.contains("belongs to another repository"),
        "{}",
        error.message
    );
    assert!(
        !error.message.contains("check the remote URL"),
        "{}",
        error.message
    );
    assert_eq!(log::read(&ledger).unwrap().0.len(), 1);
}
