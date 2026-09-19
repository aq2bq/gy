//! N-39: what a write reports as still missing, and the shape of the command
//! that could follow. One test per row of the brief's table, plus show.
use gy_ledger::{
    Actor, CriterionAdd, Decide, DecisionScope, FormatVersion, MemoryStore, NeedAdd, Node, NodeId,
    NodeKind, Operation, QuestionAdd, Relation, Repository, ReqAdd, ReqApprove, ReqCancel, ReqDone,
    ReqRevise, Store, link, show,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn approve_cmd(id: &NodeId) -> String {
    format!("req approve {id} --design … --heard-by … --evidence …")
}

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn scope() -> DecisionScope {
    DecisionScope::recorded("scope").unwrap()
}

fn need(hash: &str) -> Node {
    Node::need(id(NodeKind::Need, hash), SCOPE, DATE, "a need").unwrap()
}
fn question(hash: &str) -> Node {
    Node::question(id(NodeKind::Question, hash), SCOPE, DATE, "a question").unwrap()
}
fn decision(hash: &str) -> Node {
    Node::decision(id(NodeKind::Decision, hash), SCOPE, DATE, "d", scope()).unwrap()
}
fn criterion(hash: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "a criterion").unwrap()
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

fn decide(
    body: Option<String>,
    closes: Vec<NodeId>,
    relates: Vec<(Relation, NodeId, Option<String>)>,
) -> Decide {
    Decide {
        scope: SCOPE.into(),
        title: "a decision".into(),
        decision_scope: scope(),
        body,
        source: None,
        closes,
        relates,
    }
}

fn req_add(need: &NodeId) -> ReqAdd {
    ReqAdd {
        scope: SCOPE.into(),
        title: "a requirement".into(),
        needs: vec![need.clone()],
        relies_on: Vec::new(),
        targets: Vec::new(),
        reference: None,
        body: None,
    }
}

fn approve(id: &NodeId) -> ReqApprove {
    ReqApprove {
        id: id.clone(),
        design: "d".into(),
        heard_by: "m".into(),
        evidence: "e".into(),
    }
}

fn done(id: &NodeId) -> ReqDone {
    ReqDone {
        id: id.clone(),
        evidence: "shipped".into(),
    }
}

#[test]
fn add_operations_report_what_their_new_node_lacks() {
    let mut repo = repo();
    let ac = criterion("0001");
    let ac_id = ac.id().clone();
    let need = need("0002");
    let need_id = need.id().clone();
    seed(&mut repo, &[ac, need]);

    let added = NeedAdd {
        scope: SCOPE.into(),
        title: "a need".into(),
        targets: vec![ac_id],
        spawned_by: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(added.missing, ["a body", "a filed-as requirement"]);
    let added_id = added.id.clone().unwrap();
    assert_eq!(
        added.next,
        [
            format!("edit {added_id} --body-file … --reason …"),
            format!("req add \"<title>\" --need {added_id} …")
        ]
    );
    let text = show(&repo, &[added_id.to_string()], false).unwrap()[0].to_string();
    assert!(text.contains("missing: a body"), "{text}");

    let asked = QuestionAdd {
        scope: SCOPE.into(),
        title: "q".into(),
        decider: "m".into(),
        options: vec!["a".into(), "b".into()],
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    const NO_WAITER: &str =
        "a need that waits on it (waits-on) or a requirement that raised it (raised)";
    assert_eq!(
        asked.missing,
        ["a body (the ground for the options)", NO_WAITER]
    );
    let asked_id = asked.id.clone().unwrap();
    assert_eq!(
        asked.next,
        [
            format!("edit {asked_id} --body-file … --reason …"),
            format!("question close {asked_id} --by … --evidence …"),
            format!("decide … --closes {asked_id}"),
            format!("link <need> waits-on {asked_id}")
        ]
    );

    let criterion = CriterionAdd {
        scope: SCOPE.into(),
        title: "an AC".into(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    let coverage = "an approved requirement (targets)";
    assert_eq!(criterion.missing, ["a body (how to measure)", coverage]);
    let criterion_id = criterion.id.clone().unwrap();
    let edit = format!("edit {criterion_id} --body-file … --reason …");
    let add = format!("req add \"<title>\" --need <N> --targets {criterion_id}");
    assert_eq!(criterion.next, [edit, add]);

    let requirement = req_add(&need_id).run(&mut repo).unwrap();
    assert_eq!(
        requirement.missing,
        ["a relies-on decision", "a targets criterion", "ref"]
    );
    assert_eq!(
        requirement.next,
        [approve_cmd(&requirement.id.clone().unwrap())]
    );
}

#[test]
fn a_need_with_a_body_is_not_told_to_edit() {
    let mut repo = repo();
    let ac = criterion("0001");
    let ac_id = ac.id().clone();
    seed(&mut repo, &[ac]);

    let added = NeedAdd {
        scope: SCOPE.into(),
        title: "a need".into(),
        targets: vec![ac_id],
        spawned_by: None,
        body: Some("the body".into()),
    }
    .run(&mut repo)
    .unwrap();
    assert!(
        !added.missing.iter().any(|gap| gap == "a body"),
        "{:?}",
        added.missing
    );
    let id = added.id.clone().unwrap();
    assert_eq!(added.next, [format!("req add \"<title>\" --need {id} …")]);
}

#[test]
fn decide_reports_missing_and_suggests_a_link() {
    let mut repo = repo();
    let mut old = decision("0001");
    old.set_body("the changed part");
    let old_id = old.id().clone();
    let question = question("0002");
    let question_id = question.id().clone();
    seed(&mut repo, &[old, question]);

    let bare = decide(None, Vec::new(), Vec::new()).run(&mut repo).unwrap();
    assert_eq!(bare.missing, ["a body", "a question it closes"]);
    let bare_id = bare.id.clone().unwrap();
    assert_eq!(
        bare.next,
        [
            format!("edit {bare_id} --body-file … --reason …"),
            format!("link {bare_id} narrows <older D> --mark <text>")
        ]
    );

    let full = decide(
        Some("## Decision\nwe chose".into()),
        vec![question_id],
        vec![(Relation::Narrows, old_id, Some("changed part".into()))],
    )
    .run(&mut repo)
    .unwrap();
    assert!(full.missing.is_empty());
    assert!(full.next.is_empty());
}

#[test]
fn requirement_transitions_report_the_next_state_only() {
    let mut repo = repo();
    let need = need("0001");
    let need_id = need.id().clone();
    seed(&mut repo, &[need]);
    let requirement = req_add(&need_id).run(&mut repo).unwrap().id.unwrap();

    let approved = approve(&requirement).run(&mut repo).unwrap();
    assert!(approved.missing.is_empty());
    assert_eq!(
        approved.next,
        [format!("req done {requirement} --evidence …")]
    );

    let revised = ReqRevise {
        id: requirement.clone(),
        reason: "changed".into(),
        source: "conversation".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert!(revised.missing.is_empty());
    assert_eq!(revised.next, [approve_cmd(&requirement)]);

    let second = req_add(&need_id).run(&mut repo).unwrap().id.unwrap();
    let cancelled = ReqCancel {
        id: second,
        reason: "not needed".into(),
        source: "conversation".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert!(cancelled.missing.is_empty());
    assert!(cancelled.next.is_empty());

    approve(&requirement).run(&mut repo).unwrap();
    let done = done(&requirement).run(&mut repo).unwrap();
    assert!(done.missing.is_empty());
    assert!(done.next.is_empty());
}

#[test]
fn link_reports_nothing() {
    let mut repo = repo();
    let need = need("0001");
    let need_id = need.id().clone();
    let ac = criterion("0002");
    let ac_id = ac.id().clone();
    seed(&mut repo, &[need, ac]);

    let linked = link::Link {
        from: need_id,
        relation: Relation::Targets,
        to: ac_id,
        mark: None,
        remove: false,
    }
    .run(&mut repo)
    .unwrap();
    assert!(linked.missing.is_empty() && linked.next.is_empty());
}
