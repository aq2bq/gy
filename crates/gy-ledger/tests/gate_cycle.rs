//! n-e299, d-858d: the gate refuses a depends-on or lineage edge that would
//! close a cycle, on an ordinary write and on an undo, and leaves a ledger
//! that already holds a cycle alone.
use gy_ledger::link::Link;
use gy_ledger::{
    Actor, Decide, DecisionScope, Edit, FormatVersion, Link as ModelLink, MemoryStore, Node,
    NodeId, NodeKind, Operation, Relation, Repository, Store, Undo, now,
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
fn seed(repo: &mut Repository<MemoryStore>, node: &Node) -> NodeId {
    let node_id = node.id().clone();
    repo.transaction("seed", "test", |repo| repo.put(node))
        .unwrap();
    node_id
}
fn need(repo: &mut Repository<MemoryStore>, hash: &str) -> NodeId {
    seed(
        repo,
        &Node::need(id(NodeKind::Need, hash), SCOPE, DATE, "a need").unwrap(),
    )
}
fn decision(repo: &mut Repository<MemoryStore>, hash: &str, body: &str) -> NodeId {
    let mut node = Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        "a decision",
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap();
    node.set_body(body);
    seed(repo, &node)
}
fn link(from: &NodeId, relation: Relation, to: &NodeId, mark: Option<&str>) -> Link {
    Link {
        from: from.clone(),
        relation,
        to: to.clone(),
        mark: mark.map(str::to_string),
        remove: false,
    }
}
fn edge_count(repo: &Repository<MemoryStore>, from: &NodeId, relation: Relation) -> usize {
    repo.get(from)
        .unwrap()
        .unwrap()
        .links()
        .iter()
        .filter(|edge| !edge.reversed && edge.label == relation)
        .count()
}
fn seq(repo: &Repository<MemoryStore>) -> u64 {
    now(repo, None).unwrap().seq
}

/// A ledger that already holds a depends-on cycle, put straight into a
/// MemoryStore without the gate (the shape of an older copy).
fn seeded_cycle() -> (Repository<MemoryStore>, NodeId, NodeId) {
    let mut store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut n1 = Node::need(id(NodeKind::Need, "0001"), SCOPE, DATE, "n1").unwrap();
    let mut n2 = Node::need(id(NodeKind::Need, "0002"), SCOPE, DATE, "n2").unwrap();
    n1.link(ModelLink::new(n1.id().clone(), Relation::DependsOn, n2.id().clone()).unwrap());
    n2.link(ModelLink::new(n2.id().clone(), Relation::DependsOn, n1.id().clone()).unwrap());
    store
        .transaction(|store| {
            for node in [&n1, &n2] {
                let key = node.id().to_string();
                store.stage(key.clone(), serde_json::to_vec(node).unwrap());
                store.record(&key, "created", "seed", "test");
            }
            Ok(())
        })
        .unwrap();
    let n1 = n1.id().clone();
    let n2 = n2.id().clone();
    (Repository::new(store), n1, n2)
}

#[test]
fn depends_on_self_loops_and_cycles_are_refused_and_write_nothing() {
    let mut repo = repo();
    let n1 = need(&mut repo, "0001");
    let n2 = need(&mut repo, "0002");
    let n3 = need(&mut repo, "0003");

    // A self-loop.
    let before = seq(&repo);
    let refused = link(&n1, Relation::DependsOn, &n1, None).run(&mut repo);
    assert!(
        refused
            .as_ref()
            .is_err_and(|error| error.message.contains("would form a cycle")),
        "{refused:?}"
    );
    assert_eq!(seq(&repo), before);
    assert_eq!(edge_count(&repo, &n1, Relation::DependsOn), 0);

    // Two points.
    link(&n1, Relation::DependsOn, &n2, None)
        .run(&mut repo)
        .unwrap();
    let before = seq(&repo);
    assert!(
        link(&n2, Relation::DependsOn, &n1, None)
            .run(&mut repo)
            .is_err()
    );
    assert_eq!(seq(&repo), before);
    assert_eq!(edge_count(&repo, &n2, Relation::DependsOn), 0);

    // Three points.
    link(&n2, Relation::DependsOn, &n3, None)
        .run(&mut repo)
        .unwrap();
    assert!(
        link(&n3, Relation::DependsOn, &n1, None)
            .run(&mut repo)
            .is_err()
    );
    assert_eq!(edge_count(&repo, &n3, Relation::DependsOn), 0);
}

#[test]
fn a_lineage_self_loop_and_a_mixed_two_point_cycle_are_refused() {
    let mut repo = repo();
    let first = decision(&mut repo, "0001", "the first scope");
    let second = decision(&mut repo, "0002", "the second scope");

    // A self-loop (widens carries no mark; supersedes needs one).
    assert!(
        link(&first, Relation::Widens, &first, None)
            .run(&mut repo)
            .is_err()
    );
    assert!(
        link(
            &first,
            Relation::Supersedes,
            &first,
            Some("the first scope")
        )
        .run(&mut repo)
        .is_err()
    );

    // narrows one way, then supersedes back: one cycle from two labels.
    link(&second, Relation::Narrows, &first, Some("the first scope"))
        .run(&mut repo)
        .unwrap();
    let refused = link(
        &first,
        Relation::Supersedes,
        &second,
        Some("the second scope"),
    )
    .run(&mut repo);
    assert!(
        refused
            .as_ref()
            .is_err_and(|error| error.message.contains("would form a cycle")),
        "{refused:?}"
    );
    assert_eq!(edge_count(&repo, &first, Relation::Supersedes), 0);
}

#[test]
fn a_decision_created_by_decide_joins_the_lineage_graph() {
    let mut repo = repo();
    let older = decision(&mut repo, "0002", "the second scope");
    let third = Decide {
        scope: SCOPE.to_string(),
        title: "third".to_string(),
        decision_scope: DecisionScope::recorded("the new scope").unwrap(),
        body: None,
        source: None,
        closes: Vec::new(),
        relates: vec![(
            Relation::Supersedes,
            older.clone(),
            Some("the second scope".to_string()),
        )],
    }
    .run(&mut repo)
    .unwrap()
    .id
    .unwrap();

    // The decision decide made is a node of the graph, so closing the cycle
    // from the older side is refused.
    let refused = link(&older, Relation::Supersedes, &third, Some("the new scope")).run(&mut repo);
    assert!(
        refused
            .as_ref()
            .is_err_and(|error| error.message.contains("would form a cycle")),
        "{refused:?}"
    );
    assert_eq!(edge_count(&repo, &older, Relation::Supersedes), 0);
}

#[test]
fn a_diamond_of_edges_passes() {
    let mut repo = repo();
    let n1 = need(&mut repo, "0001");
    let n2 = need(&mut repo, "0002");
    let n3 = need(&mut repo, "0003");
    let n4 = need(&mut repo, "0004");
    for (from, to) in [(&n1, &n2), (&n1, &n3), (&n2, &n4), (&n3, &n4)] {
        link(from, Relation::DependsOn, to, None)
            .run(&mut repo)
            .unwrap();
    }
    assert_eq!(edge_count(&repo, &n4, Relation::DependsOn), 0);
}

#[test]
fn an_existing_cycle_leaves_unrelated_writes_alone() {
    let (mut repo, n1, _) = seeded_cycle();
    let before = seq(&repo);
    Edit {
        id: n1,
        reason: "tidy".to_string(),
        title: Some("renamed".to_string()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(seq(&repo), before + 1);
}

#[test]
fn an_undo_cannot_restore_an_edge_that_would_close_a_cycle() {
    let (mut repo, _, n2) = seeded_cycle();
    let n1 = id(NodeKind::Need, "0001");
    // Break the cycle.
    Link {
        from: n2.clone(),
        relation: Relation::DependsOn,
        to: n1,
        mark: None,
        remove: true,
    }
    .run(&mut repo)
    .unwrap();
    assert_eq!(edge_count(&repo, &n2, Relation::DependsOn), 0);

    // The undo would put the edge back, closing the cycle, so the gate refuses
    // it and the edge stays out.
    assert!(
        Undo {
            reason: "mistake".to_string(),
        }
        .run(&mut repo)
        .is_err()
    );
    assert_eq!(edge_count(&repo, &n2, Relation::DependsOn), 0);
}
