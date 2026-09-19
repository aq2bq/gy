//! Raising a shared ledger's format (n-96f8, ac-682e): a newer copy raises the
//! remote with one commit that changes only `format`, before any write of the
//! new rules; a copy that takes it in keeps the version. Local bare remotes.
use gy_ledger::{FormatVersion, Node, NodeId, NodeKind, format, log, sync};
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

fn bare(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "--bare"]);
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        "an ac",
    )
    .unwrap()
}

fn event(seq: u64, changes: Vec<log::Change>) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: "piko".to_string(),
        why: "a write".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: None,
        by_mail: None,
        changes,
    }
}

fn created(node: &Node) -> log::Change {
    log::Change::Created {
        node: node.id().to_string(),
        value: serde_json::to_value(node).unwrap(),
    }
}

/// A version 3 ledger with one write, pushed later by `sync`. Writing the file
/// directly keeps the copy at 3, because sync does not open the store.
fn seed(dir: &Path) -> u64 {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion(3)).unwrap();
    log::append(dir, &event(1, vec![created(&criterion("0001"))])).unwrap();
    1
}

/// The version in a bare remote's `format` at its tip.
fn tip_format(remote: &Path) -> String {
    git(remote, &["show", "main:format"])
}

/// The version in a copy's work tree.
fn read_format(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(format::FILE)).unwrap()
}

fn message(remote: &Path, revision: &str) -> String {
    git(remote, &["log", "-1", "--format=%B", revision])
}

#[test]
fn a_newer_copy_raises_the_remote_format_with_one_commit() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (remote, one) = (temp.path().join("remote.git"), temp.path().join("one"));
    bare(&remote);
    seed(&one);
    sync(&one, &url(&remote)).unwrap();
    assert_eq!(tip_format(&remote), "3\n");

    // The copy is migrated to the current format; the remote is still 3.
    format::write(&one, FormatVersion::CURRENT).unwrap();
    sync(&one, &url(&remote)).unwrap();

    assert_eq!(tip_format(&remote), "4\n");
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "2");
    let text = message(&remote, "main");
    assert!(text.contains("Gy-Format: 4"), "{text}");
    assert!(!text.contains("Gy-Seq"), "{text}");
    let files = git(
        &remote,
        &["diff-tree", "--no-commit-id", "--name-only", "-r", "main"],
    );
    assert_eq!(files.trim(), "format");
}

#[test]
fn a_copy_behind_takes_in_the_raise_and_keeps_it() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (remote, one) = (temp.path().join("remote.git"), temp.path().join("one"));
    bare(&remote);
    seed(&one);
    sync(&one, &url(&remote)).unwrap();

    // The second copy is cloned at 3 before the raise.
    let second = temp.path().join("second");
    sync(&second, &url(&remote)).unwrap();
    assert_eq!(read_format(&second), "3\n");

    format::write(&one, FormatVersion::CURRENT).unwrap();
    sync(&one, &url(&remote)).unwrap();
    assert_eq!(tip_format(&remote), "4\n");

    // Taking the raise in makes the second copy 4, and a further take-in keeps
    // it there.
    sync(&second, &url(&remote)).unwrap();
    assert_eq!(read_format(&second), "4\n");
    sync(&second, &url(&remote)).unwrap();
    assert_eq!(read_format(&second), "4\n");
}

#[test]
fn the_raise_lands_before_the_writes_of_the_new_rules() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (remote, one) = (temp.path().join("remote.git"), temp.path().join("one"));
    bare(&remote);
    seed(&one);
    sync(&one, &url(&remote)).unwrap();

    // An uncommitted write of the new rules beside the migration.
    log::append(&one, &event(2, vec![created(&criterion("0002"))])).unwrap();
    format::write(&one, FormatVersion::CURRENT).unwrap();
    sync(&one, &url(&remote)).unwrap();

    assert_eq!(tip_format(&remote), "4\n");
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "3");
    // The raise is the parent of the write, and the initial commit is below it.
    assert!(
        message(&remote, "main~0").contains("Gy-Seq: 2"),
        "{}",
        message(&remote, "main~0")
    );
    assert!(
        message(&remote, "main~1").contains("Gy-Format: 4"),
        "{}",
        message(&remote, "main~1")
    );
    assert_eq!(git(&remote, &["show", "main~1:format"]), "4\n");
    assert_eq!(git(&remote, &["show", "main~2:format"]), "3\n");
}

#[test]
fn a_copy_behind_a_moved_remote_raises_after_taking_it_in() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (remote, first) = (temp.path().join("remote.git"), temp.path().join("first"));
    bare(&remote);
    seed(&first);
    sync(&first, &url(&remote)).unwrap();

    // A copy that will be behind, already migrated to 4.
    let behind = temp.path().join("behind");
    sync(&behind, &url(&remote)).unwrap();
    format::write(&behind, FormatVersion::CURRENT).unwrap();

    // A peer moves the remote with a write, still at format 3.
    let peer = temp.path().join("peer");
    sync(&peer, &url(&remote)).unwrap();
    log::append(&peer, &event(2, vec![created(&criterion("0002"))])).unwrap();
    sync(&peer, &url(&remote)).unwrap();
    assert_eq!(tip_format(&remote), "3\n");

    // The behind copy takes the remote in (losing its 4 with the reset) and
    // then raises it back, for itself and the remote alike.
    sync(&behind, &url(&remote)).unwrap();
    assert_eq!(read_format(&behind), "4\n");
    assert_eq!(tip_format(&remote), "4\n");
    sync(&behind, &url(&remote)).unwrap();
    assert_eq!(read_format(&behind), "4\n");
}
