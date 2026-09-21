//! The ledger's name is entered by the server into the tab's title and the
//! sidebar's slot, in one last pass (n-f500).
use gy_ledger::{Actor, FormatVersion, MemoryStore, Repository};
use gy_serve::api::route;
use gy_serve::http::Request;

/// The index a server named `name` hands to a browser.
fn page(name: &str) -> String {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    let res = route(&repo, &Request::new("GET", "/", None, &[]), name);
    assert_eq!(res.status, 200);
    String::from_utf8(res.body).unwrap()
}

/// The name goes in last, and both name slots go in one pass, so a name that
/// spells any placeholder stays itself (n-f500).
#[test]
fn the_ledger_name_is_not_replaced_again() {
    for name in ["%LANG%", "%REPO%", "%NAME%", "%LEDGER%"] {
        let html = page(name);
        let small = format!("<small data-testid=\"ledger\">{name}</small>");
        assert!(html.contains(&small), "{name}: {html}");
        assert!(
            html.contains(&format!("<title>gy - {name}</title>")),
            "{name}: {html}"
        );
    }
}

/// A name may carry the characters that could close a tag or an attribute
/// (ac-c404): both slots show it escaped.
#[test]
fn the_ledger_name_is_escaped_on_the_page() {
    let html = page("we<ird&\"name");
    let escaped = "we&lt;ird&amp;&quot;name";
    assert!(
        html.contains(&format!("<small data-testid=\"ledger\">{escaped}</small>")),
        "{html}"
    );
    assert!(
        html.contains(&format!("<title>gy - {escaped}</title>")),
        "{html}"
    );
    assert!(!html.contains("we<ird"), "{html}");
}

/// Two ledgers with different names read apart on the page (ac-5d43).
#[test]
fn two_ledgers_show_their_own_names() {
    assert!(page("one").contains(r#"<small data-testid="ledger">one</small>"#));
    assert!(page("two").contains(r#"<small data-testid="ledger">two</small>"#));
}

/// No name: the tab is plain `gy` and the sidebar leaves the slot empty
/// (n-f500).
#[test]
fn an_empty_name_leaves_the_sidebar_slot_empty() {
    let html = page("");
    assert!(html.contains("<title>gy</title>"), "{html}");
    assert!(
        html.contains(r#"<small data-testid="ledger"></small>"#),
        "{html}"
    );
}
