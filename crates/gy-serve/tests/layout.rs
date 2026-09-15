use gy_ledger::link::Link;
use gy_ledger::{
    Actor, FormatVersion, MemoryStore, Node, NodeId, NodeKind, Operation, Relation, Repository,
};
use gy_serve::layout::{Graph, layout};
use std::collections::HashMap;

const DATE: &str = "2026-09-15";

fn id(hash: &str) -> NodeId {
    NodeId::from_hash(NodeKind::Need, hash).unwrap()
}

/// Two scopes with 30 nodes, 20 edges inside them, and 3 across.
fn ledger() -> Vec<Node> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("e2e").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string(), "b".to_string()]);
    let mut nodes: Vec<Node> = Vec::new();
    for index in 0..30 {
        let scope = if index < 18 { "a" } else { "b" };
        let hash = format!("0{index:03}");
        nodes.push(Node::need(id(&hash), scope, DATE, "a need").unwrap());
    }
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    // Ten edges inside each scope, and three across.
    let a = |at: usize| id(&format!("0{at:03}"));
    let b = |at: usize| id(&format!("0{:03}", 18 + at));
    let mut link = |from: NodeId, to: NodeId| {
        Link {
            from,
            relation: Relation::DependsOn,
            to,
            mark: None,
            remove: false,
        }
        .run(&mut repo)
        .unwrap();
    };
    for index in 0..10 {
        link(a(index), a(index + 1));
        link(b(index), b(index + 1));
    }
    for index in 0..3 {
        link(a(index), b(index));
    }
    // The stored nodes carry the edges; the ones built here do not.
    repo.all().unwrap()
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

#[test]
fn the_same_nodes_give_the_same_picture() {
    let nodes = ledger();
    let states = HashMap::new();
    assert_eq!(layout(&nodes, &states, 7), layout(&nodes, &states, 7));
}

#[test]
fn the_bubbles_hold_their_nodes_and_clear_each_other() {
    let nodes = ledger();
    let graph: Graph = layout(&nodes, &HashMap::new(), 7);
    assert_eq!(graph.bubbles.len(), 2);

    for (at, bubble) in graph.bubbles.iter().enumerate() {
        for other in graph.bubbles.iter().skip(at + 1) {
            let apart = distance((bubble.x, bubble.y), (other.x, other.y));
            assert!(apart > bubble.r + other.r, "{bubble:?} {other:?}");
        }
        let held = graph.nodes.iter().filter(|node| node.scope == bubble.scope);
        assert_eq!(held.clone().count(), bubble.count);
        for node in held {
            let from_center = distance((node.x, node.y), (bubble.x, bubble.y));
            assert!(from_center < bubble.r, "{node:?} {bubble:?}");
        }
    }
}

#[test]
fn the_edges_and_degrees_match_the_ledger() {
    let nodes = ledger();
    let graph = layout(&nodes, &HashMap::new(), 7);
    assert_eq!(graph.edges.len(), 20);
    assert_eq!(graph.cross.len(), 3);
    // One degree per end of every edge inside a scope.
    let degrees: usize = graph.nodes.iter().map(|node| node.degree).sum();
    assert_eq!(degrees, 40);
    let first = graph
        .nodes
        .iter()
        .find(|node| node.id == id("0000").to_string())
        .unwrap();
    assert_eq!(first.degree, 1);
    // The cross edges name both ends.
    for (from, to) in &graph.cross {
        assert!(graph.nodes.iter().any(|node| &node.id == from));
        assert!(graph.nodes.iter().any(|node| &node.id == to));
    }
}

#[test]
fn an_empty_ledger_has_an_empty_picture() {
    let graph = layout(&[], &HashMap::new(), 3);
    assert_eq!(graph.seq, 3);
    assert!(graph.bubbles.is_empty());
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
    assert!(graph.cross.is_empty());
}
