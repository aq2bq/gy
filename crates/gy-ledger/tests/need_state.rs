//! n-f60a, d-85c6: a need's done needs every criterion it targets satisfied,
//! and the criteria left behind by closed requirements show as gaps.
use gy_ledger::{
    Actor, CriterionSatisfy, FormatVersion, MemoryStore, NeedAdd, Node, NodeId, NodeKind,
    Operation, Repository, ReqAdd, ReqApprove, ReqDone, Store, advice_for, show,
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

fn seed_criterion(repo: &mut Repository<MemoryStore>, hash: &str) -> NodeId {
    let node = Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "a criterion").unwrap();
    let criterion = node.id().clone();
    repo.transaction("seed", "test", |repo| repo.put(&node))
        .unwrap();
    criterion
}

fn add_need(repo: &mut Repository<MemoryStore>, targets: &[NodeId]) -> NodeId {
    NeedAdd {
        scope: SCOPE.into(),
        title: "a need".into(),
        targets: targets.to_vec(),
        spawned_by: None,
        body: Some("the body".into()),
    }
    .run(repo)
    .unwrap()
    .id
    .unwrap()
}

fn add_request(repo: &mut Repository<MemoryStore>, need: &NodeId, targets: &[NodeId]) -> NodeId {
    ReqAdd {
        scope: SCOPE.into(),
        title: "a requirement".into(),
        needs: vec![need.clone()],
        relies_on: Vec::new(),
        targets: targets.to_vec(),
        reference: None,
        body: None,
    }
    .run(repo)
    .unwrap()
    .id
    .unwrap()
}

fn approve(repo: &mut Repository<MemoryStore>, requirement: &NodeId) {
    ReqApprove {
        id: requirement.clone(),
        design: "d".into(),
        heard_by: "m".into(),
        evidence: "e".into(),
    }
    .run(repo)
    .unwrap();
}

fn satisfy(repo: &mut Repository<MemoryStore>, criterion: &NodeId, revoke: bool) {
    CriterionSatisfy {
        id: criterion.clone(),
        evidence: "measured".into(),
        revoke,
    }
    .run(repo)
    .unwrap();
}

fn done(repo: &mut Repository<MemoryStore>, requirement: &NodeId) {
    ReqDone {
        id: requirement.clone(),
        evidence: "shipped".into(),
    }
    .run(repo)
    .unwrap();
}

/// A need and the criterion it does not yet meet: the parent targets AC1 and
/// AC2, one requirement R is approved, AC2 is satisfied, and R is done. The
/// parent derives as open, because AC1 is left.
fn scenario(repo: &mut Repository<MemoryStore>) -> (NodeId, NodeId) {
    let ac1 = seed_criterion(repo, "0001");
    let ac2 = seed_criterion(repo, "0002");
    let parent = add_need(repo, &[ac1.clone(), ac2.clone()]);
    let first = add_request(repo, &parent, std::slice::from_ref(&ac2));
    approve(repo, &first);
    satisfy(repo, &ac2, false);
    done(repo, &first);
    (parent, ac1)
}

fn state_of<S: Store>(repo: &Repository<S>, need: &NodeId) -> Option<&'static str> {
    show(repo, &[need.to_string()], false).unwrap()[0].need
}

fn advice<S: Store>(repo: &Repository<S>, need: &NodeId) -> (Vec<String>, Vec<String>) {
    let node = repo.get(need).unwrap().unwrap();
    advice_for(repo, &node).unwrap()
}

#[test]
fn a_need_becomes_done_only_when_its_targeted_criteria_are_satisfied() {
    let mut repo = repo();
    let (parent, ac1) = scenario(&mut repo);
    // ac-4d2a 1: the filed requirement is done, but AC1 is not met, so open.
    assert_eq!(state_of(&repo, &parent), Some("open"));

    let second = add_request(&mut repo, &parent, std::slice::from_ref(&ac1));
    approve(&mut repo, &second);
    satisfy(&mut repo, &ac1, false);
    done(&mut repo, &second);
    // ac-4d2a 2: every requirement done and every criterion satisfied, so done.
    assert_eq!(state_of(&repo, &parent), Some("done"));

    satisfy(&mut repo, &ac1, true);
    // ac-4d2a 3: taking the criterion back reopens the need.
    assert_eq!(state_of(&repo, &parent), Some("open"));
}

#[test]
fn a_need_with_a_filed_as_reports_the_criteria_left_behind() {
    let mut repo = repo();
    let (parent, ac1) = scenario(&mut repo);

    let (missing, next) = advice(&repo, &parent);
    assert!(
        missing.contains(&format!("unmet criterion {ac1}")),
        "{missing:?}"
    );
    assert!(
        next.contains(&format!(
            "req add \"<title>\" --need {parent} --targets {ac1}"
        )),
        "{next:?}"
    );

    // A requirement still being built (filed) bears the gap, so it is not a
    // gap: neither the missing line nor the step comes back.
    add_request(&mut repo, &parent, std::slice::from_ref(&ac1));
    let (missing, next) = advice(&repo, &parent);
    assert!(
        !missing.iter().any(|gap| gap.contains(&ac1.to_string())),
        "{missing:?}"
    );
    assert!(
        !next.iter().any(|step| step.contains(&ac1.to_string())),
        "{next:?}"
    );
}

#[test]
fn a_need_without_a_filed_as_reports_no_unmet_criterion() {
    let mut repo = repo();
    let ac = seed_criterion(&mut repo, "0001");
    let parent = add_need(&mut repo, &[ac]);

    let (missing, next) = advice(&repo, &parent);
    // A new need says it lacks a requirement, not that the criterion is left.
    assert_eq!(missing, ["a filed-as requirement"]);
    assert_eq!(next, [format!("req add \"<title>\" --need {parent} …")]);
}
