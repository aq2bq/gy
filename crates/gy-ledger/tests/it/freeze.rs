//! n-a3f2: an approved requirement freezes its title, body, and its targets and
//! relies-on edges, and freezes the title and body of a criterion it targets.
//! `req revise` is the way back. A state move, a free attribute, and a scope
//! move pass, and the rule looks only at the changed node.
use gy_ledger::link::Link as LinkOp;
use gy_ledger::{
    Actor, CriterionSatisfy, DecisionScope, Edit, FormatVersion, Link, MemoryStore, Node, NodeData,
    NodeId, NodeKind, Operation, Relation, Repository, ReqApprove, ReqCancel, ReqDone, ReqRevise,
    RequirementState,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
    .with_scopes(vec!["a".into(), "b".into()])
}

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn criterion(hash: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "a criterion").unwrap()
}

fn decision(hash: &str) -> Node {
    Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("applies").unwrap(),
    )
    .unwrap()
}

fn need(hash: &str) -> Node {
    Node::need(id(NodeKind::Need, hash), SCOPE, DATE, "a need").unwrap()
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

fn targets(node: &mut Node, criterion: &Node) {
    node.link(Link::new(node.id().clone(), Relation::Targets, criterion.id().clone()).unwrap());
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

fn approve(id: &NodeId) -> ReqApprove {
    ReqApprove {
        id: id.clone(),
        design: "design:1".into(),
        heard_by: "master".into(),
        evidence: "heard it".into(),
    }
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

fn link(from: &NodeId, relation: Relation, to: &NodeId, remove: bool) -> LinkOp {
    LinkOp {
        from: from.clone(),
        relation,
        to: to.clone(),
        mark: None,
        remove,
    }
}

fn satisfy(id: &NodeId, revoke: bool) -> CriterionSatisfy {
    CriterionSatisfy {
        id: id.clone(),
        evidence: "verified".into(),
        revoke,
    }
}

fn satisfied(node: &Node) -> bool {
    matches!(node.data(), NodeData::Criterion(data) if data.satisfied)
}

#[test]
fn an_approved_requirement_keeps_its_content_and_frozen_edges() {
    let mut repo = repo();
    let ac = criterion("0001");
    let other = criterion("0002");
    let decision = decision("0003");
    let mut request = requirement("0004", RequirementState::Filed);
    targets(&mut request, &ac);
    let id = request.id().clone();
    seed(
        &mut repo,
        &[ac.clone(), other.clone(), decision.clone(), request],
    );
    approve(&id).run(&mut repo).unwrap();

    let mut rename = edit(&id);
    rename.title = Some("a new title".into());
    let why = rename.run(&mut repo).unwrap_err().message;
    assert!(why.contains("is approved"), "{why}");
    assert!(why.contains(&format!("req revise {id}")), "{why}");

    let mut rewrite = edit(&id);
    rewrite.body = Some("a new body".into());
    assert!(rewrite.run(&mut repo).is_err());
    assert!(
        link(&id, Relation::Targets, other.id(), false)
            .run(&mut repo)
            .is_err()
    );
    assert!(
        link(&id, Relation::ReliesOn, decision.id(), false)
            .run(&mut repo)
            .is_err()
    );
    assert!(
        link(&id, Relation::Targets, ac.id(), true)
            .run(&mut repo)
            .is_err()
    );

    let mut free = edit(&id);
    free.set = vec![("owner".into(), "piko".into())];
    free.run(&mut repo).unwrap();
    let mut moved = edit(&id);
    moved.set = vec![("scope".into(), "b".into())];
    moved.run(&mut repo).unwrap();
    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.title(), "a requirement");
    assert_eq!(node.links().len(), 1);
    assert_eq!(node.scope(), "b");

    ReqRevise {
        id: id.clone(),
        reason: "changed".into(),
        source: "conversation".into(),
    }
    .run(&mut repo)
    .unwrap();
    let mut renamed = edit(&id);
    renamed.title = Some("a new title".into());
    renamed.run(&mut repo).unwrap();
    assert!(
        link(&id, Relation::Targets, other.id(), false)
            .run(&mut repo)
            .is_ok()
    );
    approve(&id).run(&mut repo).unwrap();
    let mut again = edit(&id);
    again.body = Some("again".into());
    assert!(again.run(&mut repo).is_err());
}

#[test]
fn the_other_state_moves_and_their_edits_pass() {
    let mut repo = repo();
    let done = requirement("0001", RequirementState::Filed);
    let cancelled = requirement("0002", RequirementState::Filed);
    let filed = requirement("0003", RequirementState::Filed);
    let done_id = done.id().clone();
    let cancelled_id = cancelled.id().clone();
    let filed_id = filed.id().clone();
    seed(&mut repo, &[done, cancelled, filed]);
    approve(&done_id).run(&mut repo).unwrap();
    approve(&cancelled_id).run(&mut repo).unwrap();
    ReqDone {
        id: done_id.clone(),
        evidence: "shipped".into(),
    }
    .run(&mut repo)
    .unwrap();
    ReqCancel {
        id: cancelled_id.clone(),
        reason: "not needed".into(),
        source: "conversation".into(),
    }
    .run(&mut repo)
    .unwrap();
    let mut after_done = edit(&done_id);
    after_done.body = Some("after done".into());
    after_done.run(&mut repo).unwrap();
    let mut after_cancel = edit(&cancelled_id);
    after_cancel.body = Some("after cancel".into());
    after_cancel.run(&mut repo).unwrap();
    let mut still_filed = edit(&filed_id);
    still_filed.title = Some("a filed title".into());
    still_filed.run(&mut repo).unwrap();
}

#[test]
fn the_freeze_looks_only_at_the_changed_node() {
    let mut repo = repo();
    let ac = criterion("0001");
    let mut request = requirement("0002", RequirementState::Approved);
    targets(&mut request, &ac);
    let id = request.id().clone();
    let other = need("0003");
    let other_id = other.id().clone();
    seed(&mut repo, &[ac, request, other]);

    let mut rename = edit(&other_id);
    rename.title = Some("renamed".into());
    rename.run(&mut repo).unwrap();

    let node = repo.get(&id).unwrap().unwrap();
    assert_eq!(node.state(), Some(RequirementState::Approved));
    assert_eq!(node.title(), "a requirement");
    assert_eq!(node.body(), "");
}

#[test]
fn a_criterion_is_guarded_only_by_an_approved_requirement() {
    let mut repo = repo();
    let ac = criterion("0001");
    let mut request = requirement("0002", RequirementState::Filed);
    targets(&mut request, &ac);
    let id = request.id().clone();
    seed(&mut repo, &[ac.clone(), request]);
    approve(&id).run(&mut repo).unwrap();

    let mut rename = edit(ac.id());
    rename.title = Some("renamed".into());
    let why = rename.run(&mut repo).unwrap_err().message;
    assert!(why.contains(&id.to_string()), "{why}");
    assert!(why.contains("req revise"), "{why}");
    let mut rewrite = edit(ac.id());
    rewrite.body = Some("no".into());
    assert!(rewrite.run(&mut repo).is_err());

    // Satisfying, revoking, a free attribute, and a scope move still pass.
    satisfy(ac.id(), false).run(&mut repo).unwrap();
    let mut free = edit(ac.id());
    free.set = vec![("note".into(), "x".into())];
    free.run(&mut repo).unwrap();
    let mut moved = edit(ac.id());
    moved.set = vec![("scope".into(), "b".into())];
    moved.run(&mut repo).unwrap();
    satisfy(ac.id(), true).run(&mut repo).unwrap();
    assert!(!satisfied(&repo.get(ac.id()).unwrap().unwrap()));
}

#[test]
fn a_filed_or_done_requirement_does_not_freeze_its_criterion() {
    let mut repo = repo();
    let filed_ac = criterion("0001");
    let mut filed = requirement("0002", RequirementState::Filed);
    targets(&mut filed, &filed_ac);
    let done_ac = criterion("0003");
    let mut done = requirement("0004", RequirementState::Filed);
    targets(&mut done, &done_ac);
    let done_id = done.id().clone();
    seed(&mut repo, &[filed_ac.clone(), filed, done_ac.clone(), done]);
    approve(&done_id).run(&mut repo).unwrap();
    ReqDone {
        id: done_id,
        evidence: "shipped".into(),
    }
    .run(&mut repo)
    .unwrap();

    let mut filed_rename = edit(filed_ac.id());
    filed_rename.title = Some("a filed rename".into());
    filed_rename.run(&mut repo).unwrap();
    let mut done_rename = edit(done_ac.id());
    done_rename.title = Some("a done rename".into());
    done_rename.run(&mut repo).unwrap();
}
