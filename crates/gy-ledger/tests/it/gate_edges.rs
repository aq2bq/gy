//! d-fb49: a second copy of the same forward edge is refused wherever the write
//! comes from, and a closes edge is made only when a decision closed the
//! question. A state an older copy already holds passes a write.
use gy_ledger::link::Link;
use gy_ledger::{
    Actor, Closure, Decide, DecisionScope, Edit, FormatVersion, Link as ModelLink, MemoryStore,
    NeedAdd, Node, NodeData, NodeId, NodeKind, Operation, QuestionClose, Relation, Repository,
    ReqAdd, Store, now,
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
fn seed(repo: &mut Repository<MemoryStore>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}
fn need(hash: &str) -> Node {
    Node::need(id(NodeKind::Need, hash), SCOPE, DATE, "a need").unwrap()
}
fn criterion(hash: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, "a criterion").unwrap()
}
fn decision(hash: &str) -> Node {
    let scope = DecisionScope::recorded("scope").unwrap();
    Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        "a decision",
        scope,
    )
    .unwrap()
}
fn question(hash: &str) -> Node {
    Node::question(id(NodeKind::Question, hash), SCOPE, DATE, "a question").unwrap()
}
fn link(from: &NodeId, relation: Relation, to: &NodeId) -> Link {
    Link {
        from: from.clone(),
        relation,
        to: to.clone(),
        mark: None,
        remove: false,
    }
}
fn seq(repo: &Repository<MemoryStore>) -> u64 {
    now(repo, None).unwrap().seq
}
fn forwarded(repo: &Repository<MemoryStore>, from: &NodeId, relation: Relation) -> usize {
    let node = repo.get(from).unwrap().unwrap();
    node.links()
        .iter()
        .filter(|edge| !edge.reversed && edge.label == relation)
        .count()
}
fn closure(repo: &Repository<MemoryStore>, id: &NodeId) -> Option<Closure> {
    match repo.get(id).unwrap().unwrap().data() {
        NodeData::Question(data) => data.closure,
        _ => None,
    }
}
fn registered<T>(result: &gy_ledger::Result<gy_ledger::Outcome<T>>) -> bool {
    result
        .as_ref()
        .is_err_and(|error| error.message.ends_with("is already registered"))
}
fn made_by_decision(result: &gy_ledger::Result<gy_ledger::Outcome<NodeId>>) -> bool {
    result
        .as_ref()
        .is_err_and(|error| error.message.contains("closed by decision"))
}
/// Put already-built nodes straight into a store the gate never judged, the
/// shape an older copy of the ledger could hold.
fn loose(nodes: &[&Node]) -> Repository<MemoryStore> {
    let mut store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    store
        .transaction(|store| {
            for node in nodes {
                let key = node.id().to_string();
                store.stage(key.clone(), serde_json::to_vec(node).unwrap());
                store.record(&key, "created", "seed", "test");
            }
            Ok(())
        })
        .unwrap();
    Repository::new(store)
}
fn rename(repo: &mut Repository<MemoryStore>, id: &NodeId) {
    Edit {
        id: id.clone(),
        reason: "tidy".to_string(),
        title: Some("renamed".to_string()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(repo)
    .unwrap();
}
#[test]
fn need_add_refuses_the_same_target_twice() {
    let mut repo = repo();
    let c = criterion("0001");
    seed(&mut repo, std::slice::from_ref(&c));
    let before = seq(&repo);
    let refused = NeedAdd {
        scope: SCOPE.to_string(),
        title: "a need".to_string(),
        targets: vec![c.id().clone(), c.id().clone()],
        spawned_by: None,
        body: None,
    }
    .run(&mut repo);
    assert!(registered(&refused), "{refused:?}");
    assert_eq!(seq(&repo), before);
}
#[test]
fn req_add_refuses_the_same_target_twice() {
    let mut repo = repo();
    let n = need("0001");
    let c = criterion("0002");
    seed(&mut repo, &[n.clone(), c.clone()]);
    let before = seq(&repo);
    let refused = ReqAdd {
        scope: SCOPE.to_string(),
        title: "a requirement".to_string(),
        needs: vec![n.id().clone()],
        relies_on: Vec::new(),
        targets: vec![c.id().clone(), c.id().clone()],
        reference: None,
        body: None,
    }
    .run(&mut repo);
    assert!(registered(&refused), "{refused:?}");
    assert_eq!(seq(&repo), before);
    assert_eq!(forwarded(&repo, n.id(), Relation::FiledAs), 0);
}
#[test]
fn link_refuses_a_second_copy_of_an_edge() {
    let mut repo = repo();
    let n = need("0001");
    let c = criterion("0002");
    seed(&mut repo, &[n.clone(), c.clone()]);
    link(n.id(), Relation::Targets, c.id())
        .run(&mut repo)
        .unwrap();
    let before = seq(&repo);
    let refused = link(n.id(), Relation::Targets, c.id()).run(&mut repo);
    assert!(registered(&refused), "{refused:?}");
    assert_eq!(seq(&repo), before);
    assert_eq!(forwarded(&repo, n.id(), Relation::Targets), 1);
}
#[test]
fn decide_refuses_the_same_relate_twice() {
    let mut repo = repo();
    let old = decision("0001");
    seed(&mut repo, std::slice::from_ref(&old));
    let before = seq(&repo);
    let refused = Decide {
        scope: SCOPE.to_string(),
        title: "a decision".to_string(),
        decision_scope: DecisionScope::recorded("scope").unwrap(),
        body: None,
        source: None,
        closes: Vec::new(),
        relates: vec![
            (Relation::Completes, old.id().clone(), None),
            (Relation::Completes, old.id().clone(), None),
        ],
    }
    .run(&mut repo);
    assert!(registered(&refused), "{refused:?}");
    assert_eq!(seq(&repo), before);
}
#[test]
fn an_existing_duplicate_edge_passes_a_write() {
    let c = criterion("0002");
    let mut node = need("0001");
    node.link(ModelLink::new(node.id().clone(), Relation::Targets, c.id().clone()).unwrap());
    let mut value = serde_json::to_value(&node).unwrap();
    let edges = value.get_mut("links").unwrap().as_array_mut().unwrap();
    edges.push(edges[0].clone());
    let node: Node = serde_json::from_value(value).unwrap();
    let mut repo = loose(&[&node, &c]);
    assert_eq!(forwarded(&repo, node.id(), Relation::Targets), 2);

    rename(&mut repo, node.id());
    assert_eq!(forwarded(&repo, node.id(), Relation::Targets), 2);
    NeedAdd {
        scope: SCOPE.to_string(),
        title: "another".to_string(),
        targets: vec![c.id().clone()],
        spawned_by: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap();
}
#[test]
fn question_close_refuses_a_decision_without_one() {
    for by in [Closure::Fact, Closure::NonDecision] {
        let mut repo = repo();
        let q = question("0001");
        let d = decision("0002");
        seed(&mut repo, &[q.clone(), d.clone()]);
        let before = seq(&repo);
        let refused = QuestionClose {
            id: q.id().clone(),
            by,
            evidence: "resolved".to_string(),
            decision: Some(d.id().clone()),
        }
        .run(&mut repo);
        assert!(made_by_decision(&refused), "{by:?}: {refused:?}");
        assert_eq!(seq(&repo), before);
        assert_eq!(closure(&repo, q.id()), None);
        assert_eq!(forwarded(&repo, q.id(), Relation::Closes), 0);
    }
}
#[test]
fn an_existing_non_decision_closes_passes_a_write() {
    let d = decision("0002");
    let mut q = question("0001");
    if let NodeData::Question(data) = q.data_mut() {
        data.closure = Some(Closure::Fact);
        data.evidence = Some("an older copy".to_string());
    }
    q.link(ModelLink::new(q.id().clone(), Relation::Closes, d.id().clone()).unwrap());
    let mut repo = loose(&[&q, &d]);
    assert_eq!(forwarded(&repo, q.id(), Relation::Closes), 1);

    rename(&mut repo, q.id());
    assert_eq!(closure(&repo, q.id()), Some(Closure::Fact));
    assert_eq!(forwarded(&repo, q.id(), Relation::Closes), 1);
}
