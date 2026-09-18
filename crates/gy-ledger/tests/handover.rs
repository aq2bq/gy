use gy_ledger::{
    Actor, Closure, DecisionScope, FormatVersion, Handover, Link, MemoryStore, Node, NodeData,
    NodeId, NodeKind, Ref, Relation, Repository, RequirementState, Store, handover,
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

fn seed<S: Store>(repo: &mut Repository<S>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn requirement(hash: &str, scope: &str, state: RequirementState) -> Node {
    Node::requirement(
        id(NodeKind::Requirement, hash),
        scope,
        DATE,
        "a requirement",
        state,
    )
    .unwrap()
}

fn question(hash: &str, scope: &str) -> Node {
    Node::question(id(NodeKind::Question, hash), scope, DATE, "a question").unwrap()
}

fn warning_count(report: &Handover, label: &str) -> usize {
    report
        .warnings
        .iter()
        .find(|warning| warning.label == label)
        .map_or(0, |warning| warning.count)
}

#[test]
fn handover_lists_in_progress_with_ref_and_attributes() {
    let mut repo = repo();
    let mut filed = requirement("0001", SCOPE, RequirementState::Filed);
    if let NodeData::Requirement(data) = filed.data_mut() {
        data.reference = Some(Ref("https://example/1".into()));
    }
    filed.set_free("next_evidence", "wait for the pull request");
    filed.set_free("responsible", "master");
    let approved = requirement("0002", SCOPE, RequirementState::Approved);
    let done = requirement("0003", SCOPE, RequirementState::Done);
    let cancelled = requirement("0004", SCOPE, RequirementState::Cancelled);
    seed(&mut repo, &[filed, approved, done, cancelled]);

    let report = handover(&repo, None).unwrap();
    assert!(report.errors.is_empty());
    assert_eq!(report.in_progress.len(), 2);
    let filed = report
        .in_progress
        .iter()
        .find(|row| row.id == "r-0001")
        .unwrap();
    assert_eq!(filed.reference.as_deref(), Some("https://example/1"));
    assert_eq!(filed.state, "filed");
    assert_eq!(filed.title, "a requirement");
    assert_eq!(
        filed.next_evidence.as_deref(),
        Some("wait for the pull request")
    );
    assert_eq!(filed.responsible.as_deref(), Some("master"));
}

#[test]
fn handover_counts_superseded_and_unrecorded_reliance() {
    let mut repo = repo();
    let old = Node::decision(
        id(NodeKind::Decision, "0005"),
        SCOPE,
        DATE,
        "an old decision",
        DecisionScope::recorded("old scope").unwrap(),
    )
    .unwrap();
    let mut new = Node::decision(
        id(NodeKind::Decision, "0006"),
        SCOPE,
        DATE,
        "a new decision",
        DecisionScope::recorded("new scope").unwrap(),
    )
    .unwrap();
    new.link(Link::new(new.id().clone(), Relation::Supersedes, old.id().clone()).unwrap());
    let unrecorded = Node::decision(
        id(NodeKind::Decision, "0007"),
        SCOPE,
        DATE,
        "a legacy decision",
        DecisionScope::migration_unrecorded(),
    )
    .unwrap();
    let mut first = requirement("0008", SCOPE, RequirementState::Filed);
    first.link(Link::new(first.id().clone(), Relation::ReliesOn, old.id().clone()).unwrap());
    let mut second = requirement("0009", SCOPE, RequirementState::Approved);
    second.link(
        Link::new(
            second.id().clone(),
            Relation::ReliesOn,
            unrecorded.id().clone(),
        )
        .unwrap(),
    );
    seed(&mut repo, &[old, new, unrecorded, first, second]);

    let report = handover(&repo, None).unwrap();
    assert_eq!(report.warnings.len(), 2);
    assert!(report.warnings[0].label.contains("superseded"));
    assert_eq!(report.warnings[0].count, 1);
    assert!(report.warnings[1].label.contains("unrecorded"));
    assert_eq!(report.warnings[1].count, 1);
}

#[test]
fn handover_reports_integrity_errors_and_counts_routing() {
    let mut repo = repo();
    let mut need = Node::need(id(NodeKind::Need, "0006"), SCOPE, DATE, "a need").unwrap();
    need.link(
        Link::new(
            need.id().clone(),
            Relation::FiledAs,
            id(NodeKind::Requirement, "9999"),
        )
        .unwrap(),
    );
    let criterion =
        Node::criterion(id(NodeKind::Criterion, "0007"), SCOPE, DATE, "a criterion").unwrap();
    let question =
        Node::question(id(NodeKind::Question, "0008"), SCOPE, DATE, "a question").unwrap();
    seed(&mut repo, &[need, criterion, question]);

    let report = handover(&repo, None).unwrap();
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].contains("missing"), "{:?}", report.errors);
    assert_eq!(report.open_questions, 1);
    assert_eq!(report.ready_needs, 1);
    let empty = report
        .warnings
        .iter()
        .find(|warning| warning.label.contains("empty body"))
        .unwrap();
    assert_eq!(empty.count, 1);
}

#[test]
fn handover_filters_by_scope() {
    let mut repo = repo();
    seed(
        &mut repo,
        &[
            requirement("0010", "a", RequirementState::Filed),
            requirement("0011", "b", RequirementState::Filed),
        ],
    );
    let report = handover(&repo, Some("b")).unwrap();
    assert_eq!(report.in_progress.len(), 1);
    assert_eq!(report.in_progress[0].id, "r-0011");
}

#[test]
fn handover_counts_open_questions_nobody_waits_on() {
    let mut repo = repo();
    let waited = question("0001", SCOPE);
    let raised = question("0002", SCOPE);
    let mut closed = question("0003", SCOPE);
    if let NodeData::Question(data) = closed.data_mut() {
        data.closure = Some(Closure::Fact);
    }
    let lonely = question("0004", SCOPE);
    let elsewhere = question("0005", "b");
    let mut need = Node::need(id(NodeKind::Need, "0006"), SCOPE, DATE, "a need").unwrap();
    need.link(Link::new(need.id().clone(), Relation::WaitsOn, waited.id().clone()).unwrap());
    let mut req = requirement("0007", SCOPE, RequirementState::Filed);
    req.link(Link::new(req.id().clone(), Relation::Raised, raised.id().clone()).unwrap());
    seed(
        &mut repo,
        &[waited, raised, closed, lonely, elsewhere, need, req],
    );

    let all = handover(&repo, None).unwrap();
    assert_eq!(
        warning_count(&all, "open questions nobody waits on"),
        2,
        "{:?}",
        all.warnings
    );
    let scoped = handover(&repo, Some(SCOPE)).unwrap();
    assert_eq!(warning_count(&scoped, "open questions nobody waits on"), 1);
}
