//! An edit that leaves a retracting mark unresolved is reported, not refused
//! (n-847d, d-8f76): the marks that resolved before and do not after are named
//! on `unresolved`. A mark that never resolved stays silent.
use gy_ledger::{
    Actor, DecisionScope, Edit, FormatVersion, MemoryStore, Node, NodeId, NodeKind, Operation,
    Relation, Repository, link,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn decision(hash: &str, body: &str, scope_note: &str) -> Node {
    let mut node = Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded(scope_note).unwrap(),
    )
    .unwrap();
    node.set_body(body.to_string());
    node
}

fn seed(repo: &mut Repository<MemoryStore>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

/// A newer decision that narrows `older`, quoting `mark`.
fn narrow(repo: &mut Repository<MemoryStore>, newer: &NodeId, older: &NodeId, mark: &str) {
    link::Link {
        from: newer.clone(),
        relation: Relation::Narrows,
        to: older.clone(),
        mark: Some(mark.to_string()),
        remove: false,
    }
    .run(repo)
    .unwrap();
}

fn edit(repo: &mut Repository<MemoryStore>, id: &NodeId, change: EditChange) -> Vec<String> {
    let mut request = Edit {
        id: id.clone(),
        reason: "clarify".into(),
        title: None,
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    };
    match change {
        EditChange::Body(body) => request.body = Some(body.to_string()),
        EditChange::Title(title) => request.title = Some(title.to_string()),
    }
    request.run(repo).unwrap().unresolved
}

enum EditChange {
    Body(&'static str),
    Title(&'static str),
}

fn named(by: &NodeId, mark: &str) -> String {
    format!("{by} narrows: {mark:?}")
}

#[test]
fn removing_the_marked_line_is_reported() {
    let mut repo = repo();
    let older = decision("0001", "the exact passage stays", "it holds today");
    let newer = decision("0002", "a newer call", "it holds today");
    let (older_id, newer_id) = (older.id().clone(), newer.id().clone());
    seed(&mut repo, &[older, newer]);
    narrow(&mut repo, &newer_id, &older_id, "the exact passage");

    let unresolved = edit(
        &mut repo,
        &older_id,
        EditChange::Body("a different wording"),
    );
    assert_eq!(unresolved, [named(&newer_id, "the exact passage")]);
}

#[test]
fn rewording_the_marked_line_is_reported() {
    let mut repo = repo();
    let older = decision("0001", "the exact passage stays", "it holds today");
    let newer = decision("0002", "a newer call", "it holds today");
    let (older_id, newer_id) = (older.id().clone(), newer.id().clone());
    seed(&mut repo, &[older, newer]);
    narrow(&mut repo, &newer_id, &older_id, "the exact passage");

    let unresolved = edit(
        &mut repo,
        &older_id,
        EditChange::Body("the passage, said another way"),
    );
    assert_eq!(unresolved, [named(&newer_id, "the exact passage")]);
}

#[test]
fn an_edit_that_leaves_the_mark_alone_is_silent() {
    let mut repo = repo();
    let older = decision("0001", "the exact passage stays", "it holds today");
    let newer = decision("0002", "a newer call", "it holds today");
    let (older_id, newer_id) = (older.id().clone(), newer.id().clone());
    seed(&mut repo, &[older, newer]);
    narrow(&mut repo, &newer_id, &older_id, "the exact passage");

    assert!(edit(&mut repo, &older_id, EditChange::Title("a better title")).is_empty());
    assert!(
        edit(
            &mut repo,
            &older_id,
            EditChange::Body("the exact passage still")
        )
        .is_empty()
    );
}

#[test]
fn a_mark_that_never_resolved_stays_silent() {
    let mut repo = repo();
    let older = decision("0001", "the exact passage stays", "it holds today");
    let newer = decision("0002", "a newer call", "it holds today");
    let (older_id, newer_id) = (older.id().clone(), newer.id().clone());
    seed(&mut repo, &[older, newer]);
    narrow(&mut repo, &newer_id, &older_id, "the exact passage");

    // The first edit takes the passage away and is reported.
    let first = edit(
        &mut repo,
        &older_id,
        EditChange::Body("a different wording"),
    );
    assert_eq!(first, [named(&newer_id, "the exact passage")]);

    // The mark is already unresolved: the second edit says nothing.
    assert!(edit(&mut repo, &older_id, EditChange::Title("a better title")).is_empty());
}

#[test]
fn a_mark_in_the_applicability_conditions_is_a_place_it_lives() {
    let mut repo = repo();
    let older = decision("0001", "the body text", "the applicability text");
    let newer = decision("0002", "a newer call", "it holds today");
    let (older_id, newer_id) = (older.id().clone(), newer.id().clone());
    seed(&mut repo, &[older, newer]);
    narrow(&mut repo, &newer_id, &older_id, "the applicability text");

    // The body changes; the scope note still holds the mark, so it resolves.
    assert!(edit(&mut repo, &older_id, EditChange::Body("a changed body")).is_empty());
}
