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
        "state.js",
        "events.js",
        "root.js",
        "sidebar.js",
        "topbar.js",
        "band.js",
        "now.css",
        "sky.js",
        "now.js",
        "list.css",
        "list.js",
        "eye.css",
        "eye.js",
        "history.css",
        "history.js",
        "node.css",
        "node.js",
        "palette.css",
        "palette.js",
        "time.css",
        "graph.css",
        "graph-draw.js",
        "graph.js",
        "rail.css",
        "rail.js",
        "writes.js",
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
        "state.js",
        "events.js",
        "root.js",
        "sidebar.js",
        "topbar.js",
        "band.js",
        "now.css",
        "list.css",
        "eye.css",
        "node.css",
        "palette.css",
        "sky.js",
        "now.js",
        "list.js",
        "eye.js",
        "history.js",
        "node.js",
        "palette.js",
        "time.css",
        "graph.css",
        "graph-draw.js",
        "graph.js",
        "rail.css",
        "rail.js",
        "writes.js",
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

/// The plain keys a page asks the dictionary for, as `(file, key)` pairs.
/// A template key (`st.${kind}`) is not a plain key and is skipped.
fn asked_keys() -> Vec<(&'static str, String)> {
    let mut asked = Vec::new();
    for (name, body) in assets::FILES {
        if !name.ends_with(".js") {
            continue;
        }
        /* `ui.t('x')` is how a region asks now, and the bare `t('` below skips
        it (a dot sits in front), so it is matched on its own (n-6c63). */
        for call in ["t('", "word('", "ui.t('"] {
            let mut cut = 0;
            while let Some(at) = body[cut..].find(call) {
                let start = cut + at;
                cut = start + call.len();
                let before = body[..start].chars().next_back().unwrap_or(' ');
                if before.is_alphanumeric() || before == '_' || before == '.' {
                    continue;
                }
                if let Some(end) = body[cut..].find('\'') {
                    asked.push((name, body[cut..cut + end].to_string()));
                }
            }
        }
    }
    asked
}

/// A word a page asks for by name must be in the dictionary: a missing key
/// shows its own name on the screen (found in history.js while n-b963 landed).
#[test]
fn every_plain_key_the_pages_ask_for_is_in_the_dictionary() {
    let json: Value = serde_json::from_str(assets::get("i18n.json").unwrap().0).unwrap();
    let mut known = BTreeSet::new();
    keys(&json["en"], "", &mut known);
    let asked = asked_keys();
    assert!(asked.len() > 20, "only {} keys asked", asked.len());
    for (name, key) in asked {
        assert!(
            known.contains(&key),
            "{name} asks for the missing key {key}"
        );
    }
}
