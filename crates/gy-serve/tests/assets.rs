use gy_serve::assets;
use serde_json::Value;
use std::collections::BTreeSet;

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

#[test]
fn the_read_pages_are_embedded_and_served() {
    for name in [
        "now.css",
        "now.js",
        "list.css",
        "list.js",
        "history.css",
        "history.js",
        "node.css",
        "node.js",
        "palette.css",
        "palette.js",
        "time.css",
        "time.js",
        "live.js",
        "graph.css",
        "graph-draw.js",
        "graph.js",
    ] {
        let (body, kind) = assets::get(name).unwrap();
        assert!(!body.is_empty(), "{name} is empty");
        assert!(kind.starts_with("text/"), "{name} has the type {kind}");
    }
}

#[test]
fn the_index_references_the_embedded_files() {
    let html = assets::index();
    for name in [
        "app.css",
        "now.css",
        "list.css",
        "node.css",
        "palette.css",
        "shell.js",
        "now.js",
        "list.js",
        "history.js",
        "node.js",
        "palette.js",
        "time.css",
        "time.js",
        "live.js",
        "graph.css",
        "graph-draw.js",
        "graph.js",
    ] {
        assert!(
            html.contains(&format!("assets/{name}")),
            "{name} is not linked"
        );
    }
}

#[test]
fn the_stylesheet_defines_the_frame_variables() {
    let css = assets::get("app.css").unwrap().0;
    for name in [
        "--need",
        "--question",
        "--decision",
        "--requirement",
        "--criterion",
    ] {
        assert!(css.contains(&format!("{name}:")), "{name} is not defined");
    }
    for name in ["--sans", "--mono"] {
        assert!(css.contains(&format!("{name}:")), "{name} is not defined");
    }
}

/// The dictionary's keys, nested ones flattened as `rel.closes`.
fn keys(value: &Value, prefix: &str, out: &mut BTreeSet<String>) {
    for (key, child) in value.as_object().expect("an object") {
        let path = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}.{key}")
        };
        match child {
            Value::Object(_) => keys(child, &path, out),
            _ => {
                out.insert(path);
            }
        }
    }
}

#[test]
fn the_dictionary_has_the_same_keys_in_both_languages() {
    let json: Value = serde_json::from_str(assets::get("i18n.json").unwrap().0).unwrap();
    let mut en = BTreeSet::new();
    let mut ja = BTreeSet::new();
    keys(&json["en"], "", &mut en);
    keys(&json["ja"], "", &mut ja);
    assert!(!en.is_empty());
    assert_eq!(en, ja);
}
