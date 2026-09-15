use gy_ledger::{
    Actor, Alias, Closure, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeData, NodeId,
    NodeKind, Ref, Relation, Repository, RequirementState, Store, show,
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

fn render<S: Store>(repo: &Repository<S>, texts: &[String], full: bool) -> String {
    show(repo, texts, full)
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("")
}

fn requirement(hash: &str, reference: &str) -> Node {
    let mut node = Node::requirement(
        id(NodeKind::Requirement, hash),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    if let NodeData::Requirement(data) = node.data_mut() {
        data.reference = Some(Ref(reference.into()));
    }
    node
}

#[test]
fn a_need_shows_state_and_body_without_extras() {
    let mut repo = repo();
    let mut need = Node::need(id(NodeKind::Need, "0001"), SCOPE, DATE, "a need").unwrap();
    need.set_body("need body");
    let need_id = need.id().clone();
    seed(&mut repo, &[need]);

    let text = render(&repo, &[need_id.to_string()], false);
    assert!(text.contains("a need"));
    assert!(text.contains("state: open"));
    assert!(text.contains("need body"));
    assert!(!text.contains("ref"));
    assert!(!text.contains("created:"));
    assert!(!text.contains("aliases:"));
}

#[test]
fn a_decision_shows_only_the_decision_section() {
    let mut repo = repo();
    let mut node = Node::decision(
        id(NodeKind::Decision, "0002"),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("applies at dawn").unwrap(),
    )
    .unwrap();
    node.set_body(
        "## Context\nthe context\n\n## Decision\nwe chose dawn\n\n## Consequences\nlater",
    );
    let decision_id = node.id().clone();
    seed(&mut repo, &[node]);

    let text = render(&repo, &[decision_id.to_string()], false);
    assert!(text.contains("decision_scope: applies at dawn"));
    assert!(text.contains("## Decision"));
    assert!(text.contains("we chose dawn"));
    assert!(!text.contains("the context"));
    assert!(!text.contains("later"));
}

#[test]
fn a_requirement_shows_ref_state_attributes_and_edges() {
    let mut repo = repo();
    let criterion =
        Node::criterion(id(NodeKind::Criterion, "0003"), SCOPE, DATE, "a criterion").unwrap();
    let criterion_id = criterion.id().clone();
    let mut node = requirement("0004", "https://example/6027");
    node.set_free("next_evidence", "watch the logs");
    node.set_free("responsible", "piko");
    node.link(Link::new(node.id().clone(), Relation::Targets, criterion_id.clone()).unwrap());
    let requirement_id = node.id().clone();
    seed(&mut repo, &[criterion, node]);

    let text = render(&repo, &["6027".to_string()], false);
    assert!(text.contains(&format!(
        "{requirement_id} (https://example/6027) a requirement"
    )));
    assert!(text.contains("state: filed"));
    assert!(text.contains("next_evidence: watch the logs"));
    assert!(text.contains("responsible: piko"));
    assert!(text.contains(&format!("targets {criterion_id}")));
}

#[test]
fn a_criterion_shows_satisfaction_and_a_question_shows_closure() {
    let mut repo = repo();
    let mut criterion =
        Node::criterion(id(NodeKind::Criterion, "0005"), SCOPE, DATE, "a criterion").unwrap();
    if let NodeData::Criterion(data) = criterion.data_mut() {
        data.satisfied = true;
        data.evidence = Some("verified".into());
        data.satisfied_at = Some(DATE.into());
    }
    let criterion_id = criterion.id().clone();
    let mut question =
        Node::question(id(NodeKind::Question, "0006"), SCOPE, DATE, "a question").unwrap();
    if let NodeData::Question(data) = question.data_mut() {
        data.closure = Some(Closure::Fact);
        data.decider = Some("master".into());
        data.options = vec!["a".into(), "b".into()];
    }
    let question_id = question.id().clone();
    seed(&mut repo, &[criterion, question]);

    let text = render(
        &repo,
        &[criterion_id.to_string(), question_id.to_string()],
        false,
    );
    assert!(text.contains("satisfied: true (verified) at 2026-09-15"));
    assert!(text.contains("state: closed"));
    assert!(text.contains("decider: master"));
    assert!(text.contains("options: a, b"));
}

#[test]
fn full_shows_scope_created_aliases_and_both_edge_directions() {
    let mut repo = repo();
    let criterion =
        Node::criterion(id(NodeKind::Criterion, "0007"), SCOPE, DATE, "a criterion").unwrap();
    let criterion_id = criterion.id().clone();
    let mut need = Node::need(id(NodeKind::Need, "0008"), SCOPE, DATE, "a need").unwrap();
    let mut node = Node::requirement(
        id(NodeKind::Requirement, "0009"),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    node.add_alias(Alias("old-9".into()));
    node.link(Link::new(node.id().clone(), Relation::Targets, criterion_id.clone()).unwrap());
    need.link(Link::new(need.id().clone(), Relation::FiledAs, node.id().clone()).unwrap());
    let requirement_id = node.id().clone();
    let need_id = need.id().clone();
    seed(&mut repo, &[criterion, need, node]);

    let text = render(&repo, &[requirement_id.to_string()], true);
    assert!(text.contains("scope: a"));
    assert!(text.contains("created: 2026-09-15"));
    assert!(text.contains("aliases: old-9"));
    assert!(text.contains(&format!("targets {criterion_id}")));
    assert!(text.contains(&format!("files {need_id}")));
}

#[test]
fn an_ambiguous_ref_lists_candidates_and_an_unknown_text_errors() {
    let mut repo = repo();
    let first = requirement("0010", "https://one/1");
    let second = requirement("0011", "https://two/1");
    seed(&mut repo, &[first, second]);

    let error = show(&repo, &["1".to_string()], false)
        .unwrap_err()
        .to_string();
    assert!(error.contains("matches several nodes"), "{error}");
    let error = show(&repo, &["nothing".to_string()], false)
        .unwrap_err()
        .to_string();
    assert!(error.contains("no node matches"), "{error}");
}

#[test]
fn a_shown_serializes_to_json() {
    let mut repo = repo();
    let node = Node::need(id(NodeKind::Need, "0012"), SCOPE, DATE, "a need").unwrap();
    let need_id = node.id().clone();
    seed(&mut repo, &[node]);

    let shown = show(&repo, &[need_id.to_string()], false).unwrap();
    let json = serde_json::to_string(&shown).unwrap();
    assert!(json.contains(&need_id.to_string()));
    assert!(json.contains("\"kind\":\"Need\""));
}
