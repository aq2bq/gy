use gy_ledger::{
    Actor, Closed, ClosedBy, Closure, DecisionScope, FormatVersion, Handover, Link, MemoryStore,
    Node, NodeData, NodeId, NodeKind, Ref, Relation, Repository, RequirementState, Store, handover,
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

fn need(hash: &str, scope: &str) -> Node {
    Node::need(id(NodeKind::Need, hash), scope, DATE, "a need").unwrap()
}

fn criterion(hash: &str, scope: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), scope, DATE, "a criterion").unwrap()
}

fn targets(need: &mut Node, criterion: &Node) {
    need.link(Link::new(need.id().clone(), Relation::Targets, criterion.id().clone()).unwrap());
}

fn close_need(need: &mut Node) {
    if let NodeData::Need(data) = need.data_mut() {
        data.closed = Some(Closed {
            by: ClosedBy::Fact,
            evidence: "resolved".into(),
        });
    }
}

fn satisfy(criterion: &mut Node) {
    if let NodeData::Criterion(data) = criterion.data_mut() {
        data.satisfied = true;
    }
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

const ORPHANED: &str = "criteria unmet with every need closed";

#[test]
fn handover_counts_unmet_criteria_with_every_need_closed() {
    let mut repo = repo();
    let ac = criterion("0001", SCOPE);
    let mut bearer = need("0002", SCOPE);
    targets(&mut bearer, &ac);
    close_need(&mut bearer);
    let elsewhere = criterion("0003", "b");
    let mut other = need("0004", "b");
    targets(&mut other, &elsewhere);
    close_need(&mut other);
    seed(&mut repo, &[ac, bearer, elsewhere, other]);

    let all = handover(&repo, None).unwrap();
    assert_eq!(warning_count(&all, ORPHANED), 2, "{:?}", all.warnings);
    let scoped = handover(&repo, Some(SCOPE)).unwrap();
    assert_eq!(warning_count(&scoped, ORPHANED), 1);
}

#[test]
fn handover_stops_counting_a_satisfied_criterion() {
    let mut repo = repo();
    let mut ac = criterion("0001", SCOPE);
    let mut need = need("0002", SCOPE);
    targets(&mut need, &ac);
    close_need(&mut need);
    satisfy(&mut ac);
    let mut request = requirement("0003", SCOPE, RequirementState::Approved);
    targets(&mut request, &ac);
    seed(&mut repo, &[request]);
    seed(&mut repo, &[ac, need]);
    let report = handover(&repo, None).unwrap();
    assert_eq!(warning_count(&report, ORPHANED), 0, "{:?}", report.warnings);
}

#[test]
fn handover_stops_counting_a_criterion_an_open_need_bears() {
    let mut repo = repo();
    let ac = criterion("0001", SCOPE);
    let mut closed = need("0002", SCOPE);
    targets(&mut closed, &ac);
    close_need(&mut closed);
    let mut open = need("0003", SCOPE);
    targets(&mut open, &ac);
    seed(&mut repo, &[ac, closed, open]);

    let report = handover(&repo, None).unwrap();
    assert_eq!(warning_count(&report, ORPHANED), 0, "{:?}", report.warnings);
}

#[test]
fn handover_ignores_a_criterion_no_need_targets() {
    let mut repo = repo();
    let ac = criterion("0001", SCOPE);
    let mut need = need("0002", SCOPE);
    close_need(&mut need);
    seed(&mut repo, &[ac, need]);

    let report = handover(&repo, None).unwrap();
    assert_eq!(warning_count(&report, ORPHANED), 0, "{:?}", report.warnings);
}
