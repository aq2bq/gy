//! The repository destination is entered by the server (d-4f17): the assets
//! hold the `%REPO%` placeholder, so the page a browser receives is where the
//! link can be seen. The guard in `assets.rs` keeps the raw assets free of any
//! external reference; this test and that one together say a lost insertion is
//! noticed.
use gy_ledger::{Actor, FormatVersion, MemoryStore, Repository};
use gy_serve::api::route;
use gy_serve::http::Request;

/// The index as the server hands it to a browser.
fn index() -> String {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    let res = route(&repo, &Request::new("GET", "/", None, &[]), "gy");
    assert_eq!(res.status, 200);
    String::from_utf8(res.body).unwrap()
}

#[test]
fn the_page_carries_the_repository_destination() {
    let html = index();
    assert!(
        html.contains(r#"href="https://github.com/aq2bq/gy""#),
        "the repository link is missing"
    );
    assert!(!html.contains("%REPO%"), "the placeholder is still there");
}
