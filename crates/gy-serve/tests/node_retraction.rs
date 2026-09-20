use gy_ledger::{
    Actor, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind, Relation,
    Repository,
};
use gy_serve::api::route;
use gy_serve::http::Request;

const DATE: &str = "2026-09-15T00:00:00Z";

fn id(hash: &str) -> NodeId {
    NodeId::from_hash(NodeKind::Decision, hash).unwrap()
}

fn decision(hash: &str, body: &str, scope: &str) -> Node {
    let mut node = Node::decision(
        id(hash),
        "a",
        DATE,
        "a decision",
        DecisionScope::recorded(scope).unwrap(),
    )
    .unwrap();
    node.set_body(body);
    node
}

fn answer(repo: &Repository<MemoryStore>, path: &str) -> serde_json::Value {
    let res = route(repo, &Request::new("GET", path, None, &[]), "gy");
    serde_json::from_slice(&res.body).unwrap()
}

/// A ledger where d-0002 narrows d-0001 at `mark`.
fn ledger(mark: &str, body: &str, scope: &str) -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    let older = decision("0001", body, scope);
    let mut newer = decision("0002", "body", "scope");
    newer.link(
        Link::new(newer.id().clone(), Relation::Narrows, older.id().clone())
            .unwrap()
            .with_mark(Some(mark.to_string())),
    );
    repo.transaction("seed", "test", |repo| {
        repo.put(&older)?;
        repo.put(&newer)
    })
    .unwrap();
    repo
}

#[test]
fn the_node_answer_carries_the_marked_body_beside_the_record() {
    let repo = ledger(
        "the old passage here",
        "## Decision\nthe old passage here\n",
        "applies at dawn",
    );
    let json = answer(&repo, "/api/node/d-0001");
    // The record is the full body (serve reads with --full); the marked copy
    // sits beside it, which is what the page draws.
    assert_eq!(json["body"], "## Decision\nthe old passage here\n");
    assert_eq!(
        json["body_marked"],
        "## Decision\n[[retracted by d-0002: the old passage here]]\n"
    );
    assert!(json["decision_scope_marked"].is_null());
    assert_eq!(json["cancellation"]["narrowed"][0]["by"], "d-0002");
    assert_eq!(
        json["cancellation"]["narrowed"][0]["mark"],
        "the old passage here"
    );
}

#[test]
fn the_node_answer_carries_the_marked_scope() {
    let repo = ledger("applies at dawn", "## Decision\nkeep\n", "applies at dawn");
    let json = answer(&repo, "/api/node/d-0001");
    assert_eq!(
        json["decision_scope_marked"],
        "[[retracted by d-0002: applies at dawn]]"
    );
    assert!(json["body_marked"].is_null());
    // The page reads the marked scope in place of the raw one.
    assert_eq!(json["data"]["Decision"]["scope"]["text"], "applies at dawn");
}
