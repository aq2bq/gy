//! Recovery: a damaged log, an emptied remote, a newer remote format, and
//! the way out on the second line (n-94bb 3B, ac-7fd0).
use gy_ledger::{FormatVersion, Node, NodeId, NodeKind, format, log, sync};
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn ledger(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    log::append(dir, &event(1, vec![created(&criterion("0001"))])).unwrap();
}

fn synced(temp: &Path) -> (PathBuf, PathBuf) {
    let one = temp.join("one");
    let remote = temp.join("remote.git");
    ledger(&one);
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    (one, remote)
}

#[test]
fn a_damaged_log_is_reported_without_touching_it() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, remote) = synced(temp.path());

    // A complete line that is not an event is damage.
    let path = one.join(log::FILE);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.extend_from_slice(b"not an event\n");
    std::fs::write(&path, &bytes).unwrap();
    let damaged = std::fs::read(&path).unwrap();

    let error = sync(&one, &url(&remote)).unwrap_err();
    assert!(error.message.contains("damaged"), "{}", error.message);
    assert!(
        error.message.contains("1 writes cannot be read"),
        "{}",
        error.message
    );
    assert!(
        error
            .message
            .contains("take the ledger again from the remote"),
        "{}",
        error.message
    );
    // gy never touches the damaged copy by itself.
    assert_eq!(std::fs::read(&path).unwrap(), damaged);
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "1");
}

#[test]
fn an_unterminated_tail_is_not_damage() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, remote) = synced(temp.path());
    log::append(&one, &event(2, vec![created(&criterion("0002"))])).unwrap();

    // An unterminated tail is normal; a writer drops it under the lock
    // (n-6b71), so gy sync still runs.
    let path = one.join(log::FILE);
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, &bytes[..bytes.len() - 5]).unwrap();
    assert!(sync(&one, &url(&remote)).is_ok());
}

#[test]
fn an_empty_remote_is_filled_from_this_copy() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, remote) = synced(temp.path());
    git(&remote, &["update-ref", "-d", "refs/heads/main"]);
    assert_eq!(git(&remote, &["for-each-ref", "refs/heads"]).trim(), "");

    let report = sync(&one, &url(&remote)).unwrap();
    assert!(report.reuploaded);
    assert!(
        format!("{report}").contains("re-uploaded from this copy"),
        "{report}"
    );
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "1");
}

#[test]
fn a_remote_with_a_newer_format_is_refused() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, remote) = synced(temp.path());
    let two = temp.path().join("two");
    sync(&two, &url(&remote)).unwrap();

    // The peer raises the remote's format past what this build reads.
    std::fs::write(one.join(format::FILE), "4\n").unwrap();
    git(&one, &["add", format::FILE]);
    git(&one, &["commit", "-q", "-m", "bump the format"]);
    git(&one, &["push", "-q", "origin", "HEAD:main"]);

    let error = sync(&two, &url(&remote)).unwrap_err();
    assert!(
        error.message.contains("the remote ledger is format 4"),
        "{}",
        error.message
    );
    assert!(error.message.contains("update gy"), "{}", error.message);
}

#[test]
fn an_unreachable_remote_names_the_way_out() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, remote) = synced(temp.path());
    git(
        &one,
        &[
            "remote",
            "set-url",
            "origin",
            "file:///nonexistent/ledger.git",
        ],
    );

    let error = sync(&one, &url(&remote)).unwrap_err();
    assert!(error.message.lines().count() >= 2, "{}", error.message);
    assert!(
        error.message.contains("check the remote URL"),
        "{}",
        error.message
    );
}
