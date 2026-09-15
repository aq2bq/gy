use gy_ledger::{
    Actor, Edit, FormatVersion, MemoryStore, Node, NodeId, NodeKind, Operation, Repository,
    RequirementState, Store,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
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
