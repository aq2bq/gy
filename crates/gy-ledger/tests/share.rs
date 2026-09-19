//! `gy share`'s remote check and first upload (n-57c5, ac-efd7). Local
//! `file://` remotes only; no network.
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, Operation, Repository, Rules, Share, config, format,
    share_check, share_upload,
};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

mod common;
use common::ident;

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
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

/// A local ledger with one write.
fn seed(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let mut repo = Repository::new(FileStore::open_with(dir, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: "an ac".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
}

/// A remote whose one commit holds `notes.txt`, so it is not a gy ledger.
fn non_ledger(remote: &Path) {
    bare(remote);
    let work = remote.with_extension("work");
    std::fs::create_dir_all(&work).unwrap();
    git(&work, &["init", "-q"]);
    git(&work, &["config", "user.name", "t"]);
    git(&work, &["config", "user.email", "t@e"]);
    std::fs::write(work.join("notes.txt"), "hello").unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "notes"]);
    git(&work, &["push", "-q", &url(remote), "HEAD:main"]);
}

#[test]
fn check_passes_for_an_empty_remote_and_changes_nothing() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    let remote = temp.path().join("remote.git");
    bare(&remote);

    let checked = share_check(&root, &url(&remote)).unwrap();
    assert!(checked.line.contains("is empty"), "{}", checked.line);
    assert!(!checked.branch.is_empty());
    // The scratch work happened outside the project.
    assert!(std::fs::read_dir(&root).unwrap().next().is_none());
}

#[test]
fn check_refuses_a_remote_that_is_not_a_ledger() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    let remote = temp.path().join("remote.git");
    non_ledger(&remote);
    let error = share_check(&root, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("not a gy ledger"),
        "{}",
        error.message
    );
}

#[test]
fn check_refuses_an_unreachable_remote() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let error = share_check(temp.path(), "file:///nonexistent/ledger.git").unwrap_err();
    assert!(
        error.message.contains("cannot reach the remote"),
        "{}",
        error.message
    );
}

#[test]
fn upload_pushes_the_first_copy() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("ledger");
    seed(&ledger);
    let remote = temp.path().join("remote.git");
    bare(&remote);
    let checked = share_check(&ledger, &url(&remote)).unwrap();

    let uploaded = share_upload(&ledger, &url(&remote), &checked.branch, Arc::new(Rules)).unwrap();
    assert!(
        uploaded[0].starts_with("uploaded: seq 1"),
        "{}",
        uploaded[0]
    );
    let files = git(&remote, &["ls-tree", "-r", "--name-only", "main"]);
    assert!(files.contains("events.jsonl"), "{files}");
}

#[test]
fn upload_failure_keeps_the_remote_and_names_the_way_out() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("ledger");
    seed(&ledger);
    let error = share_upload(
        &ledger,
        "file:///nonexistent/ledger.git",
        "main",
        Arc::new(Rules),
    )
    .unwrap_err();
    let wanted = "the remote stays in gy.toml; fix the access and run gy sync";
    assert!(error.message.contains(wanted), "{}", error.message);
}

#[test]
fn upload_without_a_ledger_only_says_so() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("ledger");
    let uploaded = share_upload(
        &ledger,
        "file:///nonexistent/ledger.git",
        "main",
        Arc::new(Rules),
    )
    .unwrap();
    assert!(uploaded[0].contains("no ledger yet"), "{}", uploaded[0]);
    assert!(!ledger.exists(), "no copy is made");
}

#[test]
fn write_remote_stays_top_level() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("gy.toml"), "output = \"docs\"\n\n[scopes.a]\n").unwrap();
    config::write_remote(&root, "file:///remote.git").unwrap();
    let text = std::fs::read_to_string(root.join("gy.toml")).unwrap();
    assert!(text.find("remote =").unwrap() < text.find("[scopes.").unwrap());
    assert!(text.contains("output = \"docs\""), "{text}");

    // With no scope table the line goes at the end.
    let bare_root = temp.path().join("bare");
    std::fs::create_dir_all(&bare_root).unwrap();
    std::fs::write(bare_root.join("gy.toml"), "output = \"docs\"\n").unwrap();
    config::write_remote(&bare_root, "file:///remote.git").unwrap();
    assert_eq!(
        config::read(&bare_root).unwrap().remote.as_deref(),
        Some("file:///remote.git")
    );
}

#[test]
fn the_share_lines_carry_their_prefixes_in_order() {
    ident();
    let checked = "checked: file:///remote.git is empty".to_string();
    let uploaded = vec!["uploaded: seq 1 → 1 as one commit to main".to_string()];
    let share = Share::shared("file:///remote.git", checked, uploaded);
    assert_eq!(share.lines.len(), 7);
    assert!(share.lines[0].starts_with("checked:"));
    assert!(share.lines[1].starts_with("wrote:"));
    assert!(share.lines[2].starts_with("uploaded:"));
    assert!(share.lines[6].starts_with("invite:"));
    assert_eq!(
        Share::already("file:///remote.git").lines,
        ["checked: already shared with file:///remote.git"]
    );
}
