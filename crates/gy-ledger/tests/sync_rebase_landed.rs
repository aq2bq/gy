//! A write the copy itself already pushed is not a conflict (n-6b44): the
//! rebase counts it as pushed instead of refusing it as the remote's change.
use gy_ledger::{FileStore, FormatVersion, Node, NodeId, NodeKind, Repository, format, log, sync};
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

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        id(NodeKind::Criterion, hash),
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
        why: "test".to_string(),
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

fn ledger(dir: &Path, events: &[log::Event]) {
    std::fs::create_dir_all(dir).unwrap();
    format::write(dir, FormatVersion::CURRENT).unwrap();
    for event in events {
        log::append(dir, event).unwrap();
    }
}

fn last_seq(dir: &Path) -> u64 {
    log::read(dir).unwrap().0.last().unwrap().seq
}

fn lines(dir: &Path) -> usize {
    std::fs::read_to_string(dir.join(log::FILE))
        .unwrap()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
}

fn peer_commit(dir: &Path, seq: u64, changes: Vec<log::Change>) {
    log::append(dir, &event(seq, changes)).unwrap();
    git(dir, &["add", log::FILE]);
    git(
        dir,
        &[
            "commit",
            "-q",
            "-m",
            &format!("peer\n\nactor: piko\nGy-Seq: {seq}"),
        ],
    );
    git(dir, &["push", "-q", "origin", "HEAD:main"]);
}

fn pair(temp: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let one = temp.join("one");
    let remote = temp.join("remote.git");
    ledger(&one, &[event(1, vec![created(&criterion("0001"))])]);
    bare(&remote);
    sync(&one, &url(&remote)).unwrap();
    let two = temp.join("two");
    sync(&two, &url(&remote)).unwrap();
    (one, two, remote)
}

fn store(dir: &Path) -> FileStore {
    FileStore::open_with(dir, |_| Some("piko".into())).unwrap()
}

#[test]
fn a_write_already_on_the_remote_is_counted_as_pushed() {
    ident();
    let temp = tempfile::tempdir().unwrap();
    let (one, two, remote) = pair(temp.path());
    // The copy's own write reaches the remote first: another sync of this copy
    // pushed it while this one still held it as uncommitted.
    peer_commit(&one, 2, vec![created(&criterion("0002"))]);
    log::append(&two, &event(2, vec![created(&criterion("0002"))])).unwrap();

    let report = sync(&two, &url(&remote)).unwrap();

    assert!(report.rejected.is_none(), "the landed write is not refused");
    assert!(
        !two.join("rejected.jsonl").exists(),
        "nothing is written to rejected.jsonl"
    );
    assert!(
        gy_ledger::rejected_notices(&two).is_empty(),
        "no did-not-land notice"
    );
    assert_eq!(last_seq(&two), 2);
    // Not doubled: the copy and the remote each carry the two writes once.
    assert_eq!(lines(&two), 2, "the write is not appended again");
    assert_eq!(git(&remote, &["rev-list", "--count", "main"]).trim(), "2");
    let repository = Repository::new(store(&two));
    assert!(
        repository
            .get(&id(NodeKind::Criterion, "0002"))
            .unwrap()
            .is_some(),
        "the landed write reads back from the copy"
    );
}
