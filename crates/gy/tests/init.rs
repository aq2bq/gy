//! `gy init <scope>`: start a repository where there is none, or report the one
//! already here without writing a byte (n-29b3, ac-a90b). The ledger is never
//! made here; the first write makes it (N-63).
mod common;

use common::{fixture, stderr, stdout};
use gy_ledger::{format, location};
use std::path::PathBuf;
use std::process::{Command, Output};

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

/// A nested git layout (d-0a47): `outer/gy.toml` sits above `D`, and `D`
/// carries a `.git` boundary (a directory, or a one-line file for a worktree
/// or submodule). The target is `D` itself or `D/sub`.
struct Nested {
    _temp: tempfile::TempDir,
    outer: PathBuf,
    target: PathBuf,
    data: PathBuf,
    upper_toml: String,
}

fn nested(git_file: bool, at_sub: bool) -> Nested {
    let temp = tempfile::tempdir().unwrap();
    let outer = temp.path().join("outer");
    let inner = outer.join("D");
    let target = if at_sub {
        inner.join("sub")
    } else {
        inner.clone()
    };
    std::fs::create_dir_all(&target).unwrap();
    let upper_toml = "[scopes.upper]\n".to_string();
    std::fs::write(outer.join("gy.toml"), &upper_toml).unwrap();
    if git_file {
        std::fs::write(inner.join(".git"), "gitdir: ../real.git\n").unwrap();
    } else {
        std::fs::create_dir_all(inner.join(".git")).unwrap();
    }
    let data = temp.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    Nested {
        _temp: temp,
        outer,
        target,
        data,
        upper_toml,
    }
}

impl Nested {
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_gy"))
            .current_dir(&self.target)
            .env("XDG_DATA_HOME", &self.data)
            .env("GY_ACTOR", "piko")
            .args(args)
            .output()
            .unwrap()
    }

    fn dir(&self) -> String {
        self.target.to_str().unwrap().to_string()
    }
}

#[test]
fn init_stops_at_a_git_dir_and_writes_here() {
    let fx = nested(false, true);
    let target = fx.target.clone();
    let out = fx.run(&["-C", &fx.dir(), "init", "x"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        std::fs::read_to_string(target.join("gy.toml")).unwrap(),
        "[scopes.x]\n"
    );
    let text = stdout(&out);
    assert!(text.contains("written"), "{text}");
    assert_eq!(
        std::fs::read_to_string(fx.outer.join("gy.toml")).unwrap(),
        fx.upper_toml
    );
}

#[test]
fn next_stops_at_a_git_dir_and_opens_nothing_above() {
    let fx = nested(false, true);
    let target = fx.target.clone();
    let out = fx.run(&["-C", target.to_str().unwrap(), "next"]);
    assert!(!out.status.success(), "next crossed the .git boundary");
    assert!(
        stderr(&out).contains("no gy.toml found"),
        "{}",
        stderr(&out)
    );
    assert!(
        !location::dir_in(&fx.data, &fx.outer).exists(),
        "opened the record above"
    );
}

#[test]
fn init_stops_at_a_git_file_and_writes_here() {
    let fx = nested(true, true);
    let target = fx.target.clone();
    let out = fx.run(&["-C", &fx.dir(), "init", "x"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        std::fs::read_to_string(target.join("gy.toml")).unwrap(),
        "[scopes.x]\n"
    );
    assert_eq!(
        std::fs::read_to_string(fx.outer.join("gy.toml")).unwrap(),
        fx.upper_toml
    );
}

#[test]
fn git_and_toml_in_the_same_dir_is_the_root() {
    let temp = tempfile::tempdir().unwrap();
    let inner = temp.path().join("D");
    std::fs::create_dir_all(inner.join(".git")).unwrap();
    let toml = "[scopes.inner]\n".to_string();
    std::fs::write(inner.join("gy.toml"), &toml).unwrap();
    let sub = inner.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let data = temp.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(&sub)
        .env("XDG_DATA_HOME", &data)
        .env("GY_ACTOR", "piko")
        .args(["-C", sub.to_str().unwrap(), "init", "x"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!sub.join("gy.toml").exists(), "init wrote below the root");
    let text = stdout(&out);
    assert!(text.contains("already here"), "{text}");
    assert!(
        text.contains(&inner.canonicalize().unwrap().display().to_string()),
        "{text}"
    );
    assert_eq!(
        std::fs::read_to_string(inner.join("gy.toml")).unwrap(),
        toml
    );
}
