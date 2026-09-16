//! GET /api/history against the list view: the same rows, in the same order,
//! with the filters the page sends (n-4a08).
use gy_ledger::{
    Actor, Edit, Filter, FormatVersion, Listing, MemoryStore, Node, NodeId, NodeKind, Operation,
    Repository, list,
};
use gy_serve::api::route;
use gy_serve::http::Request;

const DATE: &str = "2026-09-15";

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn criterion(hash: &str, scope: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), scope, DATE, "measures well").unwrap()
}

/// Three writes: two criteria, an edit, and one criterion in the other scope.
fn ledger() -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string(), "b".to_string()]);
    repo.transaction("seed", "test", |repo| {
        repo.put(&criterion("0001", "a"))?;
        repo.put(&criterion("0002", "a"))
    })
    .unwrap();
    Edit {
        id: id(NodeKind::Criterion, "0002"),
        reason: "tidy".to_string(),
        title: Some("measures better".to_string()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();
    repo.transaction("seed", "test", |repo| repo.put(&criterion("0003", "b")))
        .unwrap();
    repo
}

fn answer(repo: &Repository<MemoryStore>, query: Option<&str>) -> serde_json::Value {
    let res = route(repo, &Request::new("GET", "/api/history", query, &[]), "gy");
    serde_json::from_slice(&res.body).unwrap()
}

fn status(repo: &Repository<MemoryStore>, query: Option<&str>) -> u16 {
    route(repo, &Request::new("GET", "/api/history", query, &[]), "gy").status
}

#[test]
fn history_matches_the_view() {
    let repo = ledger();
    let full = answer(&repo, None);
    let view = match list(
        &repo,
        &Filter {
            since: Some(0),
            ..Default::default()
        },
    )
    .unwrap()
    {
        Listing::History(rows) => rows,
        Listing::Nodes(_) => Vec::new(),
    };
    let rows = full["rows"].as_array().unwrap();
    assert_eq!(full["total"].as_u64().unwrap() as usize, view.len());
    assert_eq!(rows.len(), view.len());
    let wanted: Vec<u64> = view.iter().rev().map(|row| row.seq).collect();
    let got: Vec<u64> = rows
        .iter()
        .map(|row| row["seq"].as_u64().unwrap())
        .collect();
    assert_eq!(got, wanted);
    assert!(
        full["actors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a == "piko")
    );

    // One writer, or none; a broken sequence is refused.
    assert_eq!(answer(&repo, Some("actor=piko"))["total"], view.len());
    assert_eq!(answer(&repo, Some("actor=nobody"))["total"], 0);
    assert_eq!(status(&repo, Some("since=bad")), 400);

    // A scope keeps its own writes only, and a point keeps the later ones.
    let scoped = answer(&repo, Some("scope=b"));
    assert_eq!(scoped["total"], 1);
    assert!(scoped["rows"].as_array().unwrap().len() < rows.len());
    let last = view.last().unwrap().seq;
    let since = answer(&repo, Some(&format!("since={}", last - 1)));
    assert_eq!(since["total"], 1);
}

/// The `limit` count caps the rows, keeps the total, and takes the default 200
/// for anything outside 1..=200 (n-b963).
#[test]
fn limit_caps_rows_and_keeps_the_total() {
    let repo = ledger();
    let full = answer(&repo, None);
    let total = full["total"].as_u64().unwrap() as usize;
    assert!(total > 2, "{total}");
    let capped = answer(&repo, Some("limit=2"));
    assert_eq!(capped["total"], full["total"]);
    assert_eq!(capped["rows"].as_array().unwrap().len(), 2);
    for query in ["limit=0", "limit=x", "limit=999"] {
        let rows = answer(&repo, Some(query));
        assert_eq!(rows["rows"].as_array().unwrap().len(), total);
    }
}
