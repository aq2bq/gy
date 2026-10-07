use gy_ledger::{
    Actor, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind, Relation,
    Repository, Shown, Store, show,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn id(hash: &str) -> NodeId {
    NodeId::from_hash(NodeKind::Decision, hash).unwrap()
}

fn decision(hash: &str, body: &str, scope: &str) -> Node {
    let mut node = Node::decision(
        id(hash),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded(scope).unwrap(),
    )
    .unwrap();
    node.set_body(body);
    node
}

fn seed<S: Store>(repo: &mut Repository<S>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn shown<S: Store>(repo: &Repository<S>, text: &str, full: bool) -> Shown {
    show(repo, &[text.to_string()], full).unwrap().remove(0)
}

fn render<S: Store>(repo: &Repository<S>, text: &str, full: bool) -> String {
    shown(repo, text, full).to_string()
}

/// A newer decision that narrows `older` at `mark`.
fn narrower(older: &Node, hash: &str, mark: &str) -> Node {
    let mut node = decision(hash, "body", "scope");
    node.link(
        Link::new(node.id().clone(), Relation::Narrows, older.id().clone())
            .unwrap()
            .with_mark(Some(mark.to_string())),
    );
    node
}

#[test]
fn a_narrowed_passage_is_marked_where_it_stands_and_named_at_the_head() {
    let mut repo = repo();
    let older = decision("0001", "## Decision\nthe old passage here\n", "scope");
    let newer = narrower(&older, "0002", "the old passage here");
    seed(&mut repo, &[older, newer]);

    let text = render(&repo, "d-0001", false);
    assert!(
        text.contains("[[retracted by d-0002: the old passage here]]"),
        "{text}"
    );
    assert!(
        text.contains("narrowed-by d-0002 (the old passage here)"),
        "{text}"
    );
}

#[test]
fn the_raw_body_stays_and_the_marked_copy_sits_beside_it() {
    let mut repo = repo();
    let older = decision("0001", "## Decision\nthe old passage here\n", "scope");
    let newer = narrower(&older, "0002", "the old passage here");
    seed(&mut repo, &[older, newer]);

    let node = shown(&repo, "d-0001", false);
    // `body` is the record as before this change: the `## Decision` section for
    // a default show, and never carrying the retraction mark.
    assert_eq!(node.body, "## Decision\nthe old passage here");
    assert!(!node.body.contains("[["));
    assert!(
        node.body_marked
            .as_deref()
            .unwrap()
            .contains("[[retracted by d-0002: the old passage here]]")
    );
    assert!(node.cancellation.unwrap().narrowed[0].by == "d-0002");
}

#[test]
fn a_superseded_decision_keeps_its_body_and_only_names_the_head() {
    let mut repo = repo();
    let older = decision("0001", "## Decision\nstill here\n", "scope");
    let mut newer = decision("0002", "body", "scope");
    newer.link(
        Link::new(newer.id().clone(), Relation::Supersedes, older.id().clone())
            .unwrap()
            .with_mark(Some("still here".to_string())),
    );
    seed(&mut repo, &[older, newer]);

    let node = shown(&repo, "d-0001", false);
    assert!(node.body_marked.is_none(), "{:?}", node.body_marked);
    let text = node.to_string();
    assert!(text.contains("superseded-by d-0002"), "{text}");
    assert!(text.contains("still here"), "{text}");
    assert_eq!(
        node.cancellation.as_ref().unwrap().superseded_by,
        vec!["d-0002".to_string()]
    );
}

#[test]
fn a_mark_in_the_applicability_conditions_marks_the_scope_line() {
    let mut repo = repo();
    let older = decision("0001", "## Decision\nkeep\n", "applies at dawn");
    let newer = narrower(&older, "0002", "applies at dawn");
    seed(&mut repo, &[older, newer]);

    let node = shown(&repo, "d-0001", false);
    assert!(node.body_marked.is_none(), "{:?}", node.body_marked);
    assert_eq!(
        node.decision_scope_marked.as_deref(),
        Some("[[retracted by d-0002: applies at dawn]]")
    );
    let text = node.to_string();
    assert!(
        text.contains("decision_scope: [[retracted by d-0002: applies at dawn]]"),
        "{text}"
    );
}

#[test]
fn a_mark_in_a_section_the_default_hides_shows_the_whole_body() {
    let mut repo = repo();
    let older = decision(
        "0001",
        "## Context\nthe old passage here\n\n## Decision\nkeep\n",
        "scope",
    );
    let newer = narrower(&older, "0002", "the old passage here");
    seed(&mut repo, &[older, newer]);

    let text = render(&repo, "d-0001", false);
    assert!(
        text.contains("[[retracted by d-0002: the old passage here]]"),
        "{text}"
    );
    // The fall-back is the whole body, so the context holding the passage shows.
    assert!(text.contains("## Context"), "{text}");
    assert!(text.contains("keep"), "{text}");
}

#[test]
fn a_decision_with_no_retraction_still_hides_the_other_sections() {
    let mut repo = repo();
    let older = decision(
        "0001",
        "## Context\nthe context\n\n## Decision\nkeep\n",
        "scope",
    );
    seed(&mut repo, &[older]);

    let text = render(&repo, "d-0001", false);
    assert!(text.contains("keep"), "{text}");
    assert!(!text.contains("the context"), "{text}");
}

#[test]
fn the_longest_mark_wins_when_one_passage_contains_another() {
    let mut repo = repo();
    let older = decision("0001", "## Decision\nabcdef\n", "scope");
    let long = narrower(&older, "0002", "abc");
    let short = narrower(&older, "0003", "ab");
    seed(&mut repo, &[older, long, short]);

    let marked = shown(&repo, "d-0001", false).body_marked.unwrap();
    assert!(
        marked.contains("[[retracted by d-0002: abc]]def"),
        "{marked}"
    );
    assert!(!marked.contains("[[retracted by d-0003: ab]]"), "{marked}");
}

#[test]
fn every_occurrence_of_a_passage_is_marked() {
    let mut repo = repo();
    let older = decision("0001", "## Decision\ndup\ndup\n", "scope");
    let newer = narrower(&older, "0002", "dup");
    seed(&mut repo, &[older, newer]);

    let marked = shown(&repo, "d-0001", false).body_marked.unwrap();
    assert_eq!(marked.matches("[[retracted by d-0002: dup]]").count(), 2);
}

#[test]
fn full_keeps_the_mark_and_still_shows_the_edge() {
    let mut repo = repo();
    let older = decision("0001", "## Decision\nthe old passage here\n", "scope");
    let newer = narrower(&older, "0002", "the old passage here");
    seed(&mut repo, &[older, newer]);

    let node = shown(&repo, "d-0001", true);
    assert!(node.body_marked.is_some());
    assert!(node.edges.iter().any(|edge| edge.name == "narrowed-by"));
}
