use gy_ledger::{
    Actor, ClosedBy, CriterionSatisfy, DecisionScope, FormatVersion, MemoryStore, NeedClose, Node,
    NodeId, NodeKind, Operation, Repository,
};
use gy_serve::api::route;
use gy_serve::http::Request;
use serde_json::json;

const DATE: &str = "2026-09-15";

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

/// Two needs (one closed), one question, one decision, two criteria (one
/// satisfied), in two scopes.
fn ledger() -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".into(), "b".into()]);
    let note = DecisionScope::recorded("s").unwrap();
    let nodes = [
        Node::need(id(NodeKind::Need, "0001"), "a", DATE, "a need").unwrap(),
        Node::need(id(NodeKind::Need, "0002"), "b", DATE, "a need").unwrap(),
        Node::question(id(NodeKind::Question, "0003"), "a", DATE, "a question").unwrap(),
        Node::decision(
            id(NodeKind::Decision, "0004"),
            "a",
            DATE,
            "a decision",
            note,
        )
        .unwrap(),
        Node::criterion(id(NodeKind::Criterion, "0005"), "b", DATE, "met").unwrap(),
        Node::criterion(id(NodeKind::Criterion, "0006"), "a", DATE, "unmet").unwrap(),
    ];
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    let (closed, met) = (id(NodeKind::Need, "0002"), id(NodeKind::Criterion, "0005"));
    let close = NeedClose {
        id: closed,
        by: ClosedBy::Fact,
        evidence: "x".into(),
    };
    let satisfy = CriterionSatisfy {
        id: met,
        evidence: "x".into(),
        revoke: false,
    };
    close.run(&mut repo).unwrap();
    satisfy.run(&mut repo).unwrap();
    repo
}

fn get(path: &str, query: Option<&str>) -> Request {
    Request::new("GET", path, query, &[])
}

fn shell(query: Option<&str>) -> serde_json::Value {
    let res = route(&ledger(), &get("/api/shell", query));
    assert_eq!(res.status, 200);
    serde_json::from_slice(&res.body).unwrap()
}

#[test]
fn shell_counts_every_kind_and_scope() {
    let json = shell(None);
    assert_eq!(json["scope"], "all");
    assert!(json["seq"].as_u64().unwrap() >= 1);
    assert!(json["at"].as_str().unwrap().starts_with("20"));
    assert_eq!(
        json["kinds"],
        json!([
            {"kind": "Need", "open": 1, "total": 2},
            {"kind": "Question", "open": 1, "total": 1},
            {"kind": "Decision", "open": 0, "total": 1},
            {"kind": "Requirement", "open": 0, "total": 0},
            {"kind": "Criterion", "open": 1, "total": 2},
        ])
    );
    assert_eq!(
        json["scopes"],
        json!([{"name": "a", "count": 4}, {"name": "b", "count": 2}])
    );
}

#[test]
fn shell_filters_by_scope() {
    let json = shell(Some("scope=b"));
    assert_eq!(json["scope"], "b");
    let kinds = json["kinds"].as_array().unwrap();
    let sum = |key: &str| {
        kinds
            .iter()
            .map(|row| row[key].as_u64().unwrap())
            .sum::<u64>()
    };
    assert_eq!((sum("open"), sum("total")), (0, 2));
    assert_eq!(json["scopes"].as_array().unwrap().len(), 2);
}

#[test]
fn other_methods_are_not_allowed() {
    let res = route(&ledger(), &Request::new("POST", "/api/shell", None, &[]));
    assert_eq!(res.status, 405);
}

#[test]
fn a_traversal_or_unknown_path_is_not_found() {
    for path in ["/assets/../Cargo.toml", "/assets/missing.js", "/nope"] {
        assert_eq!(route(&ledger(), &get(path, None)).status, 404, "{path}");
    }
}

#[test]
fn the_index_language_follows_accept_language() {
    let index = |lang: &str| {
        let headers = [("accept-language".to_string(), lang.to_string())];
        let res = route(&ledger(), &Request::new("GET", "/", None, &headers));
        String::from_utf8(res.body).unwrap()
    };
    assert!(index("ja,en;q=0.9").contains(r#"lang="ja""#));
    assert!(index("en-US").contains(r#"lang="en""#));
}
