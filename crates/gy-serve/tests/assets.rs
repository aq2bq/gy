use gy_serve::assets;

#[test]
fn embedded_files_have_no_external_reference() {
    for (name, body) in assets::FILES {
        for needle in ["http://", "https://", "//"] {
            assert!(!body.contains(needle), "{name} contains {needle}");
        }
    }
}

#[test]
fn a_traversal_or_unknown_name_is_not_a_file() {
    for name in ["../Cargo.toml", "sub/app.css", "missing.js"] {
        assert!(assets::get(name).is_none(), "{name}");
    }
    let kind = assets::get("index.html").unwrap().1;
    assert_eq!(kind, "text/html; charset=utf-8");
}
