//! A write's writer reads as the human before the agent, and `--actor`
//! matches either (n-d36d, ac-11b2).
use gy_ledger::{
    FileStore, Filter, FormatVersion, Listing, Node, NodeId, NodeKind, Repository,
    RequirementState, format, handover, list, log, now,
};
use std::path::Path;

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn event(seq: u64, actor: &str, by: Option<&str>, changes: Vec<log::Change>) -> log::Event {
    log::Event {
        seq,
        at: 0,
        actor: actor.to_string(),
        why: "a write".to_string(),
        source: "test".to_string(),
        retries: 0,
        by: by.map(str::to_string),
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

fn repo(dir: &Path) -> Repository<FileStore> {
    Repository::new(FileStore::open_with(dir, |_| Some("lead".into())).unwrap())
}

fn criterion() -> Node {
    Node::criterion(
        id(NodeKind::Criterion, "0001"),
        "a",
        "2026-09-15T00:00:00Z",
        "an ac",
    )
    .unwrap()
}

fn requirement() -> Node {
    Node::requirement(
        id(NodeKind::Requirement, "0002"),
        "a",
        "2026-09-15T00:00:00Z",
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap()
}

fn history(repo: &Repository<FileStore>) -> Vec<gy_ledger::LogRow> {
    match list(
        repo,
        &Filter {
            since: Some(0),
            ..Default::default()
        },
    )
    .unwrap()
    {
        Listing::History(rows) => rows,
        Listing::Nodes(_) => Vec::new(),
    }
}

#[test]
fn a_history_row_and_the_filter_name_the_human_first() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    ledger(
        dir,
        &[event(
            1,
            "lead",
            Some("pememo"),
            vec![created(&criterion())],
        )],
    );
    let repo = repo(dir);

    let rows = history(&repo);
    assert_eq!(rows[0].who, "pememo / lead");
    assert_eq!(rows[0].actor, "lead");

    let count = |actor: &str| match list(
        &repo,
        &Filter {
            actor: Some(actor.into()),
            since: Some(0),
            ..Default::default()
        },
    )
    .unwrap()
    {
        Listing::History(rows) => rows.len(),
        Listing::Nodes(_) => 0,
    };
    assert_eq!(count("pememo"), 1);
    assert_eq!(count("lead"), 1);
    assert_eq!(count("other"), 0);
}

#[test]
fn a_local_history_row_reads_as_the_actor() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    ledger(dir, &[event(1, "lead", None, vec![created(&criterion())])]);

    let repo = repo(dir);
    let rows = history(&repo);
    assert_eq!(rows[0].who, "lead");
    assert_eq!(now(&repo, None).unwrap().recent[0].who, "lead");
}

#[test]
fn handover_names_the_requirements_last_writer() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    ledger(
        dir,
        &[
            event(1, "piko", None, vec![created(&requirement())]),
            event(
                2,
                "lead",
                Some("pememo"),
                vec![log::Change::Updated {
                    node: "r-0002".into(),
                    value: serde_json::to_value(requirement()).unwrap(),
                }],
            ),
        ],
    );
    let report = handover(&repo(dir), None).unwrap();
    let row = &report.in_progress[0];
    assert_eq!(row.who.as_deref(), Some("pememo / lead"));
}
