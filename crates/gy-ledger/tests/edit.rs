use gy_ledger::{
    Actor, DecisionScope, Edit, FormatVersion, Link, MemoryStore, Node, NodeData, NodeId, NodeKind,
    Operation, Relation, Repository, RequirementState, Store, handover,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn repo_with(scopes: &[&str]) -> Repository<MemoryStore> {
    repo().with_scopes(scopes.iter().map(|scope| (*scope).to_string()).collect())
}

fn requirement(repo: &mut Repository<MemoryStore>) -> NodeId {
    let node = Node::requirement(
        NodeId::from_hash(NodeKind::Requirement, "0001").unwrap(),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    let id = node.id().clone();
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    id
}

fn edit(id: &NodeId) -> Edit {
    Edit {
        id: id.clone(),
        reason: "clarify".into(),
        title: None,
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
}

#[test]
fn edit_changes_title_and_body_and_leaves_the_state() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = edit(&id);
    request.title = Some("a better title".into());
    request.body = Some("the body".into());
    let outcome = request.run(&mut repo).unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.title(), "a better title");
    assert_eq!(node.body(), "the body");
    assert_eq!(node.state(), Some(RequirementState::Filed));
    assert_eq!(outcome.changed, ["title", "body"]);
}

#[test]
fn edit_sets_then_appends_free_attributes() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut first = edit(&id);
    first.set = vec![("owner".into(), "piko".into())];
    first.append = vec![("note".into(), "one".into())];
    first.run(&mut repo).unwrap();
    let mut second = edit(&id);
    second.append = vec![("note".into(), "two".into())];
    let outcome = second.run(&mut repo).unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.free("owner").map(String::as_str), Some("piko"));
    assert_eq!(node.free("note").map(String::as_str), Some("one\ntwo"));
    assert_eq!(outcome.changed, ["free:note"]);
}

#[test]
fn edit_rejects_an_empty_reason() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = edit(&id);
    request.reason = "  ".into();
    request.title = Some("x".into());
    assert!(request.run(&mut repo).is_err());
}

#[test]
fn edit_rejects_a_reserved_key() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = edit(&id);
    request.set = vec![("status".into(), "done".into())];
    assert!(request.run(&mut repo).is_err());
    assert_eq!(repo.get(&id).unwrap().unwrap().free("status"), None);
}

#[test]
fn edit_rejects_a_no_op_and_a_missing_node() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    assert!(edit(&id).run(&mut repo).is_err());
    let missing = NodeId::from_hash(NodeKind::Need, "0002").unwrap();
    let mut request = edit(&missing);
    request.title = Some("x".into());
    assert!(request.run(&mut repo).is_err());
}

#[test]
fn edit_moves_a_node_to_a_declared_scope_and_keeps_its_id_and_edges() {
    let mut repo = repo_with(&["a", "b"]);
    let criterion = Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, "0009").unwrap(),
        SCOPE,
        DATE,
        "an AC",
    )
    .unwrap();
    let criterion_id = criterion.id().clone();
    let mut node = Node::requirement(
        NodeId::from_hash(NodeKind::Requirement, "0001").unwrap(),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    node.link(Link::new(node.id().clone(), Relation::Targets, criterion_id.clone()).unwrap());
    let id = node.id().clone();
    repo.transaction("seed", "test", |repo| {
        repo.put(&criterion)?;
        repo.put(&node)
    })
    .unwrap();

    let mut request = edit(&id);
    request.set = vec![("scope".into(), "b".into())];
    let outcome = request.run(&mut repo).unwrap();

    let moved = repo.get(&id).unwrap().unwrap();
    assert_eq!(moved.scope(), "b");
    assert_eq!(moved.id(), &id);
    assert_eq!(outcome.changed, ["scope"]);
    assert_eq!(moved.links().len(), 1);
    assert_eq!(moved.links()[0].name(), "targets");
    assert_eq!(moved.links()[0].to, criterion_id);
    assert_eq!(repo.get(&criterion_id).unwrap().unwrap().scope(), "a");
}

#[test]
fn edit_rejects_an_undeclared_scope() {
    let mut repo = repo_with(&["a"]);
    let id = requirement(&mut repo);
    let mut request = edit(&id);
    request.set = vec![("scope".into(), "b".into())];
    assert!(request.run(&mut repo).is_err());
    assert_eq!(repo.get(&id).unwrap().unwrap().scope(), "a");
}

#[test]
fn edit_with_no_declared_scopes_rejects_a_move() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = edit(&id);
    request.set = vec![("scope".into(), "a".into())];
    assert!(request.run(&mut repo).is_err());
}

fn decision(repo: &mut Repository<MemoryStore>, hash: &str, unrecorded: bool) -> NodeId {
    let scope = if unrecorded {
        DecisionScope::migration_unrecorded()
    } else {
        DecisionScope::recorded("applies").unwrap()
    };
    let node = Node::decision(
        NodeId::from_hash(NodeKind::Decision, hash).unwrap(),
        SCOPE,
        DATE,
        "a decision",
        scope,
    )
    .unwrap();
    let id = node.id().clone();
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    id
}

#[test]
fn edit_records_an_unrecorded_decision_scope_once() {
    let mut repo = repo();
    let id = decision(&mut repo, "0003", true);
    let mut request = edit(&id);
    request.set = vec![("decision_scope".into(), "applies at dawn".into())];
    let outcome = request.run(&mut repo).unwrap();
    assert_eq!(outcome.changed, ["decision_scope"]);
    match repo.get(&id).unwrap().unwrap().data() {
        NodeData::Decision(data) => {
            assert!(!data.scope.is_unrecorded());
            assert_eq!(data.scope.text(), "applies at dawn");
        }
        _ => panic!("not a decision"),
    }

    let mut again = edit(&id);
    again.set = vec![("decision_scope".into(), "another".into())];
    assert!(again.run(&mut repo).is_err());
}

#[test]
fn edit_rejects_bad_scope_records() {
    let mut repo = repo();
    let recorded = decision(&mut repo, "0003", false);
    let mut request = edit(&recorded);
    request.set = vec![("decision_scope".into(), "x".into())];
    assert!(request.run(&mut repo).is_err());

    let unrecorded = decision(&mut repo, "0004", true);
    let mut request = edit(&unrecorded);
    request.set = vec![("decision_scope".into(), String::new())];
    assert!(request.run(&mut repo).is_err());
}

#[test]
fn edit_drops_a_free_attribute_with_an_empty_value() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut add = edit(&id);
    add.set = vec![("owner".into(), "piko".into())];
    add.run(&mut repo).unwrap();

    let mut remove = edit(&id);
    remove.set = vec![
        ("owner".into(), String::new()),
        ("missing".into(), String::new()),
    ];
    let outcome = remove.run(&mut repo).unwrap();
    assert_eq!(outcome.changed, ["free:owner"]);
    assert_eq!(repo.get(&id).unwrap().unwrap().free("owner"), None);
}

#[test]
fn edit_rejects_an_empty_append() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = edit(&id);
    request.append = vec![("note".into(), String::new())];
    assert!(request.run(&mut repo).is_err());
}

#[test]
fn recording_a_scope_clears_the_handover_warning() {
    let mut repo = repo();
    let decision = decision(&mut repo, "0003", true);
    let requirement = requirement(&mut repo);
    let mut node = repo.get(&requirement).unwrap().unwrap();
    node.link(Link::new(requirement.clone(), Relation::ReliesOn, decision.clone()).unwrap());
    repo.transaction("link", "test", |repo| repo.put(&node))
        .unwrap();

    let before = handover(&repo, None).unwrap();
    assert!(
        before
            .warnings
            .iter()
            .any(|warning| warning.label.contains("unrecorded") && warning.count == 1),
        "{:?}",
        before.warnings
    );

    let mut request = edit(&decision);
    request.set = vec![("decision_scope".into(), "applies at dawn".into())];
    request.run(&mut repo).unwrap();

    let after = handover(&repo, None).unwrap();
    assert!(
        after
            .warnings
            .iter()
            .all(|warning| !warning.label.contains("unrecorded")),
        "{:?}",
        after.warnings
    );
}

#[test]
fn edit_keeps_the_why_and_source() {
    let mut repo = repo();
    let id = requirement(&mut repo);
    let mut request = edit(&id);
    request.body = Some("changed".into());
    request.run(&mut repo).unwrap();
    let entry = repo.store().history().last().unwrap();
    assert_eq!(entry.why, format!("edit {id}"));
    assert_eq!(entry.source, "clarify");
}
