//! /api/now's rows name the writer too; a local ledger reads the actor
//! (n-d36d).
use gy_ledger::{Actor, CriterionAdd, FormatVersion, MemoryStore, Operation, Repository};
use gy_serve::api::route;
use gy_serve::http::Request;

fn now(repo: &Repository<MemoryStore>) -> serde_json::Value {
    let res = route(repo, &Request::new("GET", "/api/now", None, &[]), "gy");
    serde_json::from_slice(&res.body).unwrap()
}

#[test]
fn the_recent_rows_name_the_writer() {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    CriterionAdd {
        scope: "a".to_string(),
        title: "an ac".to_string(),
        body: None,
    }
    .run(&mut repo)
    .unwrap();

    let value = now(&repo);
    let rows = value["recent"].as_array().unwrap();
    assert!(!rows.is_empty());
    assert!(
        rows.iter().all(|row| row["who"] == row["actor"]),
        "{rows:?}"
    );
}
