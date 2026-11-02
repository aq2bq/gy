//! Deterministic UI load fixture, built through the model and transaction gate.
//! Run only against the fresh temporary repository supplied by the E2E helper.
use gy_ledger::{
    DecisionScope, FileStore, FormatVersion, Link, Node, NodeData, NodeId, NodeKind, Relation,
    Repository, format, location,
};
use std::path::PathBuf;

const DATE: &str = "2026-10-30T00:00:00Z";

fn id(kind: NodeKind, n: usize) -> NodeId {
    NodeId::from_hash(kind, format!("{n:04x}")).unwrap()
}

fn aim(node: &mut Node, relation: Relation, target: NodeId) {
    node.link(Link::new(node.id().clone(), relation, target).unwrap());
}

fn nodes() -> Vec<Node> {
    let criterion =
        Node::criterion(id(NodeKind::Criterion, 1), "a", DATE, "the big criterion").unwrap();
    let mut nodes = vec![criterion];
    for n in 0..41 {
        let (scope, title) = if n == 40 {
            ("b", "the big need".into())
        } else {
            ("a", format!("need {n}"))
        };
        let mut node = Node::need(id(NodeKind::Need, n), scope, DATE, &title).unwrap();
        aim(&mut node, Relation::Targets, id(NodeKind::Criterion, 1));
        nodes.push(node);
    }
    for n in 0..60 {
        let mut node = Node::decision(
            id(NodeKind::Decision, n),
            "b",
            DATE,
            &format!("decision {n}"),
            DecisionScope::recorded("load fixture").unwrap(),
        )
        .unwrap();
        aim(&mut node, Relation::Serves, id(NodeKind::Need, 40));
        nodes.push(node);
    }
    // Match the observed kind proportions, with 31 creation dates. Scopes
    // remain 900 / 901 / 369 to exercise both sides of the rendering cap.
    for n in 0..2068 {
        let scope = match n {
            0..859 => "a",
            859..1699 => "b",
            _ => "c",
        };
        let date = format!("2026-10-{:02}T00:00:00Z", n % 31 + 1);
        let hash = n + 100;
        let mut node = match n {
            0..566 => Node::need(
                id(NodeKind::Need, hash),
                scope,
                &date,
                &format!("need {hash}"),
            ),
            566..1250 => {
                Node::criterion(id(NodeKind::Criterion, hash), scope, &date, "a criterion")
            }
            1250..1693 => Node::decision(
                id(NodeKind::Decision, hash),
                scope,
                &date,
                &format!("decision {hash}"),
                DecisionScope::recorded("load fixture").unwrap(),
            ),
            1693..1895 => Node::requirement(
                id(NodeKind::Requirement, hash),
                scope,
                &date,
                "a requirement",
                gy_ledger::RequirementState::Filed,
            ),
            _ => Node::question(id(NodeKind::Question, hash), scope, &date, "a question"),
        }
        .unwrap();
        match node.kind() {
            NodeKind::Need | NodeKind::Requirement => {
                aim(&mut node, Relation::Targets, id(NodeKind::Criterion, 1))
            }
            NodeKind::Decision => aim(&mut node, Relation::Serves, id(NodeKind::Need, 40)),
            _ => {}
        }
        if let NodeData::Question(data) = node.data_mut() {
            data.decider = Some("lead".into());
            data.options = vec!["one".into(), "two".into()];
        }
        nodes.push(node);
    }
    nodes
}

fn main() {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("temporary repository root"),
    );
    let data = PathBuf::from(std::env::var_os("XDG_DATA_HOME").expect("isolated data directory"));
    let dir = location::dir_in(&data, &root.canonicalize().unwrap());
    assert!(!dir.exists(), "fixture must not overwrite a ledger");
    std::fs::create_dir_all(&dir).unwrap();
    format::write(&dir, FormatVersion::CURRENT).unwrap();
    let mut repo = Repository::new(FileStore::open(&dir).unwrap());
    let nodes = nodes();
    assert_eq!(nodes.len(), 2170);
    // Each batch is a normal gated transaction, with history for every node.
    for batch in nodes.chunks(100) {
        repo.transaction("load fixture", "synthetic UI test", |repo| {
            for node in batch {
                repo.put(node)?;
            }
            Ok(())
        })
        .unwrap();
    }
    assert_eq!(repo.all().unwrap().len(), 2170);
}
