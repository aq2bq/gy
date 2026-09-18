//! A ledger directory inside another work tree (n-6d6c, ac-ad40): gy makes it
//! its own nested repository and never touches the parent, and a copy whose
//! `origin` is not its `remote` is refused. No network.
use gy_ledger::{
    CriterionAdd, FileStore, FormatVersion, Operation, Range, Repository, format, log, sync,
};
use std::path::Path;
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

/// A bare remote that stays unborn.
fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

/// A local ledger with one write.
fn seed(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    write_criterion(dir, "an ac");
}

/// One more criterion write; enough to make the working log ahead of HEAD.
fn add_write(dir: &Path) {
    write_criterion(dir, "another");
}

fn write_criterion(dir: &Path, title: &str) {
    let mut repo = Repository::new(FileStore::open_with(dir, |_| Some("piko".into())).unwrap());
    CriterionAdd {
        scope: "a".into(),
        title: title.into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
}

/// A parent work tree whose origin is another repository, as a home directory
/// kept in dotfiles is.
fn outer_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q"]);
    git(
        dir,
        &[
            "remote",
            "add",
            "origin",
            "file:///nonexistent/dotfiles.git",
        ],
    );
    std::fs::write(dir.join("bashrc"), "export EDITOR=vi\n").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "dotfiles"]);
}

/// The parent work tree's HEAD and index, as text to compare across a sync.
fn parent_state(dir: &Path) -> (String, String) {
    (
        git(dir, &["rev-parse", "HEAD"]),
        git(dir, &["ls-files", "-s"]),
    )
}

/// The ledger is its own repository and the remote holds the ledger, not the
/// parent.
fn assert_nested_copy(ledger: &Path, remote: &Path) {
    assert!(ledger.join(".git").is_dir(), "no nested .git");
    assert_eq!(
        git(ledger, &["remote", "get-url", "origin"]).trim(),
        url(remote)
    );
    assert_eq!(
        std::fs::read_to_string(ledger.join("remote")).unwrap(),
        url(remote)
    );
    let files = git(remote, &["ls-tree", "-r", "--name-only", "main"]);
    assert!(files.contains("events.jsonl"), "{files}");
    assert!(!files.contains("bashrc"), "{files}");
}

#[test]
fn a_ledger_inside_another_work_tree_becomes_its_own_repo() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let outer = temp.path().join("dotfiles");
    outer_repo(&outer);
    // The ledger sits under the parent's work tree, as `~/.local/share/gy` does
    // under a home directory kept in git.
    let ledger = outer.join("local/share/gy/fae59914");
    seed(&ledger);
    let remote = temp.path().join("remote.git");
    bare(&remote);
    let before = parent_state(&outer);

    let report = sync(&ledger, &url(&remote)).unwrap();
    assert_eq!(report.pushed, Some(Range { from: 1, to: 1 }));
    assert_eq!(parent_state(&outer), before, "the parent repo changed");
    assert_nested_copy(&ledger, &remote);

    // A later write still pushes through the nested copy, and the parent stays
    // untouched.
    add_write(&ledger);
    let seq = log::read(&ledger).unwrap().0.last().unwrap().seq;
    sync(&ledger, &url(&remote)).unwrap();
    assert!(
        git(&remote, &["log", "-1", "--format=%B", "main"]).contains(&format!("Gy-Seq: {seq}"))
    );
    assert_eq!(
        parent_state(&outer),
        before,
        "the parent repo changed again"
    );
}

#[test]
fn a_copy_whose_origin_is_not_its_remote_is_refused() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (ledger, a, b) = (
        temp.path().join("ledger"),
        temp.path().join("a.git"),
        temp.path().join("b.git"),
    );
    seed(&ledger);
    bare(&a);
    bare(&b);
    sync(&ledger, &url(&a)).unwrap();

    // Point `origin` at a different repository while the marker still names a.
    git(&ledger, &["remote", "set-url", "origin", &url(&b)]);
    add_write(&ledger);

    let error = sync(&ledger, &url(&a)).unwrap_err();
    assert!(
        error.message.contains("belongs to another repository"),
        "{}",
        error.message
    );
    // Nothing landed in the wrong remote.
    assert!(git(&b, &["rev-list", "--all"]).trim().is_empty());
}
