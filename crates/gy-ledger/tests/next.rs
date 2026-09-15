use gy_ledger::{
    Actor, Closed, ClosedBy, Closure, FormatVersion, Link, MemoryStore, NextRow, Node, NodeData,
    NodeId, NodeKind, Ref, Relation, Repository, RequirementState, Store, next,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
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

fn need(hash: &str, scope: &str, title: &str) -> Node {
    Node::need(id(NodeKind::Need, hash), scope, DATE, title).unwrap()
}

fn criterion(hash: &str, satisfied: bool) -> Node {
    let mut node =
        Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "a criterion").unwrap();
    if let NodeData::Criterion(data) = node.data_mut() {
        data.satisfied = satisfied;
    }
    node
}

fn requirement(hash: &str, state: RequirementState) -> Node {
    Node::requirement(
        id(NodeKind::Requirement, hash),
        SCOPE,
        DATE,
        "a requirement",
        state,
    )
    .unwrap()
}

fn close_need(need: &mut Node) {
    if let NodeData::Need(data) = need.data_mut() {
        data.closed = Some(Closed {
            by: ClosedBy::Fact,
            evidence: "done".into(),
        });
    }
}

fn ids(rows: &[NextRow]) -> Vec<String> {
    rows.iter().map(|row| row.id.clone()).collect()
}

#[test]
fn next_skips_closed_and_done_needs_and_counts_targets() {
    let mut repo = repo();
    let criterion = criterion("0001", true);
    let criterion_id = criterion.id().clone();
    let mut open = need("0002", SCOPE, "open need");
    open.link(Link::new(open.id().clone(), Relation::Targets, criterion_id.clone()).unwrap());

    let mut closed = need("0003", SCOPE, "closed need");
    close_need(&mut closed);

    let done_requirement = requirement("0004", RequirementState::Done);
    let mut done = need("0005", SCOPE, "done need");
    done.link(
        Link::new(
            done.id().clone(),
            Relation::FiledAs,
            done_requirement.id().clone(),
        )
        .unwrap(),
    );
    seed(
        &mut repo,
        &[criterion, open, closed, done_requirement, done],
    );

    let rows = next(&repo, None).unwrap();
    assert_eq!(ids(&rows), ["n-0002"]);
    assert_eq!(rows[0].satisfied, 1);
    assert_eq!(rows[0].targets, 1);
}

#[test]
fn next_lists_filed_requirements_with_ref_and_state() {
    let mut repo = repo();
    let mut requirement = requirement("0006", RequirementState::Filed);
    if let NodeData::Requirement(data) = requirement.data_mut() {
        data.reference = Some(Ref("https://example/6".into()));
    }
    let requirement_id = requirement.id().clone();
    let mut need = need("0007", SCOPE, "with requirement");
    need.link(Link::new(need.id().clone(), Relation::FiledAs, requirement_id.clone()).unwrap());
    seed(&mut repo, &[requirement, need]);

    let rows = next(&repo, None).unwrap();
    assert_eq!(ids(&rows), ["n-0007"]);
    assert_eq!(rows[0].requirements.len(), 1);
    assert_eq!(rows[0].requirements[0].id, requirement_id.to_string());
    assert_eq!(
        rows[0].requirements[0].reference.as_deref(),
        Some("https://example/6")
    );
    assert_eq!(rows[0].requirements[0].state, "filed");
}

#[test]
fn next_waits_for_dependencies() {
    let mut repo = repo();
    let dependency = need("0008", SCOPE, "dependency");
    let dependency_id = dependency.id().clone();
    let mut blocked = need("0009", SCOPE, "blocked");
    blocked.link(
        Link::new(
            blocked.id().clone(),
            Relation::DependsOn,
            dependency_id.clone(),
        )
        .unwrap(),
    );
    seed(&mut repo, &[dependency, blocked]);
    assert_eq!(ids(&next(&repo, None).unwrap()), ["n-0008"]);

    let mut closed = repo.get(&dependency_id).unwrap().unwrap();
    close_need(&mut closed);
    seed(&mut repo, &[closed]);
    assert_eq!(ids(&next(&repo, None).unwrap()), ["n-0009"]);
}

#[test]
fn next_waits_for_questions_in_waits_on() {
    let mut repo = repo();
    let question =
        Node::question(id(NodeKind::Question, "0010"), SCOPE, DATE, "a question").unwrap();
    let question_id = question.id().clone();
    let mut need = need("0011", SCOPE, "waiting");
    need.link(Link::new(need.id().clone(), Relation::WaitsOn, question_id.clone()).unwrap());
    seed(&mut repo, &[question, need]);
    assert!(next(&repo, None).unwrap().is_empty());

    let mut closed = repo.get(&question_id).unwrap().unwrap();
    if let NodeData::Question(data) = closed.data_mut() {
        data.closure = Some(Closure::Fact);
    }
    seed(&mut repo, &[closed]);
    assert_eq!(ids(&next(&repo, None).unwrap()), ["n-0011"]);
}

#[test]
fn next_waits_for_a_requirement_in_waits_on() {
    let mut repo = repo();
    let requirement = requirement("0012", RequirementState::Filed);
    let requirement_id = requirement.id().clone();
    let mut need = need("0013", SCOPE, "waiting on a requirement");
    need.link(Link::new(need.id().clone(), Relation::WaitsOn, requirement_id.clone()).unwrap());
    seed(&mut repo, &[requirement, need]);
    assert!(next(&repo, None).unwrap().is_empty());

    let mut done = repo.get(&requirement_id).unwrap().unwrap();
    if let NodeData::Requirement(data) = done.data_mut() {
        data.state = RequirementState::Done;
    }
    seed(&mut repo, &[done]);
    assert_eq!(ids(&next(&repo, None).unwrap()), ["n-0013"]);
}

#[test]
fn next_filters_by_scope() {
    let mut repo = repo();
    seed(
        &mut repo,
        &[need("0012", "a", "in a"), need("0013", "b", "in b")],
    );
    assert_eq!(ids(&next(&repo, Some("a")).unwrap()), ["n-0012"]);
    assert_eq!(ids(&next(&repo, Some("b")).unwrap()), ["n-0013"]);
}
