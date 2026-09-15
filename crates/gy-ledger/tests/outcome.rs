//! N-39: what a write reports as still missing, and the shape of the command
//! that could follow. One test per row of the brief's table, plus show.
use gy_ledger::{
    Actor, CriterionAdd, CriterionSatisfy, Decide, DecisionScope, Edit, FormatVersion, Link,
    MemoryStore, NeedAdd, Node, NodeId, NodeKind, Operation, QuestionAdd, Relation, Repository,
    ReqAdd, ReqApprove, ReqCancel, ReqDone, ReqRevise, Store, Undo, link, show,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";
const APPROVE: &str = "req approve <ID> --design … --heard-by … --evidence …";

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
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(added.missing, ["本文", "filed-as の要求"]);
    assert_eq!(
        added.next,
        [
            "edit <ID> --body-file … --reason …",
            "req add \"<題>\" --need <ID> …"
        ]
    );
    let text = show(&repo, &[added.id.unwrap().to_string()], false).unwrap()[0].to_string();
    assert!(text.contains("無いもの: 本文, filed-as の要求"), "{text}");

    let asked = QuestionAdd {
        scope: SCOPE.into(),
        title: "q".into(),
        decider: "m".into(),
        options: vec!["a".into(), "b".into()],
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(asked.missing, ["本文（選択肢の根拠）"]);
    assert_eq!(
        asked.next,
        [
            "question close <ID> --by … --evidence …",
            "decide … --closes <ID>"
        ]
    );

    let criterion = CriterionAdd {
        scope: SCOPE.into(),
        title: "an AC".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(criterion.missing, ["本文（測り方）"]);
    assert_eq!(criterion.next, ["criterion satisfy <AC> --evidence …"]);

    let requirement = req_add(&need_id).run(&mut repo).unwrap();
    assert_eq!(
        requirement.missing,
        ["relies-on の決定", "targets の AC", "ref"]
    );
    assert_eq!(requirement.next, [APPROVE]);
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
    assert_eq!(bare.missing, ["本文", "closes した論点"]);
    assert_eq!(bare.next, ["link <D> narrows <古い D> --mark <文>"]);

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
    assert_eq!(approved.next, ["req done <ID> --evidence …"]);

    let revised = ReqRevise {
        id: requirement.clone(),
        reason: "changed".into(),
        source: "conversation".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert!(revised.missing.is_empty());
    assert_eq!(revised.next, [APPROVE]);

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
fn criterion_satisfy_points_at_the_other_criterion() {
    let mut repo = repo();
    let need = need("0001");
    let need_id = need.id().clone();
    let first = criterion("0002");
    let first_id = first.id().clone();
    let second = criterion("0003");
    let second_id = second.id().clone();
    let mut need = need;
    need.link(Link::new(need_id.clone(), Relation::Targets, first_id.clone()).unwrap());
    need.link(Link::new(need_id, Relation::Targets, second_id.clone()).unwrap());
    seed(&mut repo, &[need, first, second]);

    let satisfied = CriterionSatisfy {
        id: first_id,
        evidence: "verified".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    assert!(satisfied.missing.is_empty());
    assert_eq!(satisfied.next, ["criterion satisfy <他の AC>"]);

    let last = CriterionSatisfy {
        id: second_id,
        evidence: "verified".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    assert!(last.next.is_empty());
}

#[test]
fn link_edit_and_undo_report_nothing() {
    let mut repo = repo();
    let need = need("0001");
    let need_id = need.id().clone();
    let ac = criterion("0002");
    let ac_id = ac.id().clone();
    seed(&mut repo, &[need, ac]);

    let linked = link::Link {
        from: need_id.clone(),
        relation: Relation::Targets,
        to: ac_id,
        mark: None,
        remove: false,
    }
    .run(&mut repo)
    .unwrap();
    assert!(linked.missing.is_empty() && linked.next.is_empty());

    let edited = Edit {
        id: need_id,
        reason: "r".into(),
        title: Some("t".into()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();
    assert!(edited.missing.is_empty() && edited.next.is_empty());

    let undone = Undo {
        reason: "mistake".into(),
    }
    .run(&mut repo)
    .unwrap();
    assert!(undone.missing.is_empty() && undone.next.is_empty());
}
