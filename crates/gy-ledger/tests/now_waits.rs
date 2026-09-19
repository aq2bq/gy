//! The `now` queue reads a person from the ledger's writers (d-b02d): a decider
//! that never wrote waits, and the git `by` name is not a writer. The other
//! `now` readings live in `now.rs`.
use gy_ledger::{
    Actor, FileStore, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Operation,
    QuestionAdd, Repository, format, now,
};

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn q(hash: &str) -> String {
    id(NodeKind::Question, hash).to_string()
}

/// One node as one write, so `created` is ours and the write count is exact.
fn put(repo: &mut Repository<MemoryStore>, node: &Node) {
    repo.transaction("seed", "test", |repo| repo.put(node))
        .unwrap();
}

/// A question with a decider and two options, as one write.
fn ask(
    repo: &mut Repository<MemoryStore>,
    hash: &str,
    scope: &str,
    created: &str,
    decider: &str,
) -> NodeId {
    let id = id(NodeKind::Question, hash);
    let mut node = Node::question(id.clone(), scope, created, "a question").unwrap();
    if let NodeData::Question(data) = node.data_mut() {
        data.decider = Some(decider.to_string());
        data.options = vec!["x".to_string(), "y".to_string()];
    }
    put(repo, &node);
    id
}

/// Four questions written by `lead`, one per decider, so the writer set is
/// exactly `lead`.
fn deciders() -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("lead").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    ask(&mut repo, "0101", "a", "2026-09-01T00:00:00Z", "alice");
    ask(&mut repo, "0102", "a", "2026-09-02T00:00:00Z", "lead");
    ask(&mut repo, "0103", "a", "2026-09-03T00:00:00Z", "master");
    ask(&mut repo, "0104", "a", "2026-09-04T00:00:00Z", "マスター");
    repo
}

fn ids(rows: &[gy_ledger::NodeRow]) -> Vec<String> {
    rows.iter().map(|row| row.id.clone()).collect()
}

/// The question rows in the wait queue, in order.
fn waiting_ids(view: &gy_ledger::Now) -> Vec<String> {
    view.waiting
        .iter()
        .filter_map(|item| match item {
            gy_ledger::Waiting::Question { row, .. } => Some(row.id.clone()),
            gy_ledger::Waiting::Requirement { .. } => None,
        })
        .collect()
}

/// A decider that never wrote waits (a); one that wrote is an agent's (b); and
/// the old `master` / `マスター` expectations do not change (c).
#[test]
fn now_waits_on_a_decider_that_never_wrote() {
    let repo = deciders();
    let view = now(&repo, None).unwrap();
    assert_eq!(waiting_ids(&view), [q("0101"), q("0103"), q("0104")]);
    assert_eq!(ids(&view.open_questions), [q("0102")]);
}

/// No ambient git identity: the test sets exactly what it wants.
fn hermetic_git() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
    });
}

fn git(cwd: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
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

/// A copy with a format file and the `remote` marker, so a write records `by`
/// from git's configured name (n-8a52).
fn shared_copy(where_: &std::path::Path) -> std::path::PathBuf {
    let dir = where_.join("copy");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    git(&dir, &["init", "-q"]);
    std::fs::write(dir.join("remote"), "file:///example/ledger.git").unwrap();
    git(&dir, &["config", "user.name", "aq2bq"]);
    git(&dir, &["config", "user.email", "aq2bq@example.com"]);
    dir
}

/// A name that only the git `by` carries is not a writer, so its question
/// still waits (d).
#[test]
fn now_does_not_count_the_git_name_as_a_writer() {
    hermetic_git();
    let temp = tempfile::tempdir().unwrap();
    let dir = shared_copy(temp.path());
    let mut repo = Repository::new(FileStore::open_with(&dir, |_| Some("piko".into())).unwrap())
        .with_scopes(vec!["a".to_string()]);
    let mut added = Vec::new();
    for decider in ["aq2bq", "piko"] {
        let id = QuestionAdd {
            scope: "a".into(),
            title: format!("a question for {decider}"),
            decider: decider.into(),
            options: vec!["x".into(), "y".into()],
            body: None,
        }
        .run(&mut repo)
        .unwrap()
        .value;
        added.push(id.to_string());
    }
    let view = now(&repo, None).unwrap();
    let waiting: Vec<String> = view
        .waiting
        .iter()
        .filter_map(|item| match item {
            gy_ledger::Waiting::Question { decider, .. } => Some(decider.clone()),
            gy_ledger::Waiting::Requirement { .. } => None,
        })
        .collect();
    assert_eq!(waiting, ["aq2bq"], "the git name is not a writer");
    assert_eq!(ids(&view.open_questions), [added[1].clone()]);
}
