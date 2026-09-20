use gy_ledger::link::Link;
use gy_ledger::{
    Actor, EGO_LIMIT, FormatVersion, MemoryStore, Node, NodeId, NodeKind, Operation, Relation,
    Repository, ego,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

fn id(hash: &str) -> NodeId {
    NodeId::from_hash(NodeKind::Need, hash).unwrap()
}

/// A chain of needs, each depending on the next.
fn chain(count: usize) -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store);
    let nodes: Vec<Node> = (0..count)
        .map(|index| Node::need(id(&format!("{index:04}")), SCOPE, DATE, "a need").unwrap())
        .collect();
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    for index in 0..count.saturating_sub(1) {
        Link {
            from: id(&format!("{index:04}")),
            relation: Relation::DependsOn,
            to: id(&format!("{:04}", index + 1)),
            mark: None,
            remove: false,
        }
        .run(&mut repo)
        .unwrap();
    }
    repo
}

#[test]
fn ego_walks_to_the_depth_and_keeps_the_near() {
    let repo = chain(4);
    let hood = ego(&repo, "n-0000", 2).unwrap();
    assert_eq!(hood.root, "n-0000");
    let reached: Vec<(&str, usize)> = hood
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.hop))
        .collect();
    assert_eq!(
        reached,
        vec![("n-0000", 0), ("n-0001", 1), ("n-0002", 2)],
        "depth is the ring, not the whole reachable set"
    );
    assert_eq!(hood.truncated, 0);
    assert_eq!(hood.edges.len(), 2);
    assert_eq!(hood.edges[0].name, "depends-on");
    assert_eq!(hood.nodes[1].kind, Some(NodeKind::Need));
    assert_eq!(hood.nodes[1].title.as_deref(), Some("a need"));
}

#[test]
fn ego_depth_one_stops_at_the_direct_edges() {
    let repo = chain(3);
    let hood = ego(&repo, "n-0000", 1).unwrap();
    let reached: Vec<&str> = hood.nodes.iter().map(|node| node.id.as_str()).collect();
    assert_eq!(reached, vec!["n-0000", "n-0001"]);
    assert_eq!(hood.edges.len(), 1);
}

#[test]
fn ego_caps_at_the_limit_and_counts_what_it_left_out() {
    let repo = chain(45);
    let hood = ego(&repo, "n-0000", 50).unwrap();
    assert_eq!(hood.nodes.len(), EGO_LIMIT);
    assert_eq!(hood.truncated, 45 - EGO_LIMIT);
    assert_eq!(hood.nodes[0].hop, 0, "the focus stays, nearest first");
}
