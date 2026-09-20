//! `gy init <scope>`: start a repository where there is none, or report the one
//! already here without writing a byte (n-29b3, ac-a90b). The ledger is never
//! made here; the first write makes it (N-63).
mod common;

use common::{fixture, stderr, stdout};
use gy_ledger::format;

/// A repository root whose gy.toml has been removed, so init has work to do.
fn no_toml() -> common::Fixture {
    let fx = fixture();
    std::fs::remove_file(fx.root.join("gy.toml")).unwrap();
    fx
}

#[test]
fn init_writes_gy_toml_and_names_the_first_node() {
    let fx = no_toml();
    let out = fx.run(&["init", "myproject"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        std::fs::read_to_string(fx.root.join("gy.toml")).unwrap(),
        "[scopes.myproject]\n"
    );
    let text = stdout(&out);
    assert!(text.contains("gy.toml written"), "{text}");
    assert!(text.contains("myproject"), "{text}");
    assert!(text.contains("gy-loop"), "{text}");
    assert!(text.contains("GY_ACTOR"), "{text}");
    assert!(text.contains("gy criterion add"), "{text}");
}

#[test]
fn init_creates_no_ledger() {
    let fx = no_toml();
    let out = fx.run(&["init", "myproject"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        !fx.ledger().join(format::FILE).is_file(),
        "init made a ledger"
    );
}

#[test]
fn init_is_idempotent() {
    let fx = no_toml();
    let first = fx.run(&["init", "a"]);
    assert!(first.status.success(), "{}", stderr(&first));
    let before = std::fs::read_to_string(fx.root.join("gy.toml")).unwrap();
    let second = fx.run(&["init", "a"]);
    assert!(second.status.success(), "{}", stderr(&second));
    let text = stdout(&second);
    assert!(text.contains("already here"), "{text}");
    assert!(text.contains("Nothing was written"), "{text}");
    assert_eq!(
        std::fs::read_to_string(fx.root.join("gy.toml")).unwrap(),
        before
    );
}

#[test]
fn init_reports_scopes_and_remote_when_one_is_here() {
    let fx = fixture();
    let toml = "remote = \"https://example.invalid/l.git\"\n\n[scopes.a]\n[scopes.b]\n";
    std::fs::write(fx.root.join("gy.toml"), toml).unwrap();
    let out = fx.run(&["init", "ignored"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("already here"), "{text}");
    assert!(text.contains("scope: a, b"), "{text}");
    assert!(
        text.contains("remote: https://example.invalid/l.git"),
        "{text}"
    );
    assert_eq!(
        std::fs::read_to_string(fx.root.join("gy.toml")).unwrap(),
        toml
    );
}

#[test]
fn init_walks_up_and_never_writes_below() {
    let fx = fixture();
    let sub = fx.root.join("deep/inside");
    std::fs::create_dir_all(&sub).unwrap();
    let out = fx.run(&["-C", sub.to_str().unwrap(), "init", "x"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!sub.join("gy.toml").exists(), "init wrote below the root");
    let text = stdout(&out);
    assert!(text.contains("already here"), "{text}");
    let root = fx.root.canonicalize().unwrap();
    assert!(text.contains(&root.display().to_string()), "{text}");
}

#[test]
fn init_creates_where_the_directory_is_named() {
    let fx = fixture();
    let elsewhere = fx.root.parent().unwrap().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let out = fx.run(&["-C", elsewhere.to_str().unwrap(), "init", "fresh"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        std::fs::read_to_string(elsewhere.join("gy.toml")).unwrap(),
        "[scopes.fresh]\n"
    );
}

#[test]
fn init_refuses_a_name_that_is_not_a_bare_key() {
    let fx = no_toml();
    let out = fx.run(&["init", "bad name"]);
    assert_eq!(out.status.code(), Some(2));
    let text = stderr(&out);
    assert!(text.contains("scope names take A-Za-z0-9_- only"), "{text}");
    assert!(!fx.root.join("gy.toml").exists(), "gy.toml was written");
}

#[test]
fn init_json_reports_the_same() {
    let fx = no_toml();
    let out = fx.run(&["--json", "init", "myproject"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(value["created"], true);
    assert_eq!(value["scopes"][0], "myproject");
    assert!(value["remote"].is_null(), "{value}");
}
