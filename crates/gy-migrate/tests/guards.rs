use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    _temp: tempfile::TempDir,
    ledger: PathBuf,
    root: PathBuf,
    data: PathBuf,
}

fn node(id: &str, kind: &str, extra: &str) -> String {
    format!(
        "---\nid: {id}\ntype: {kind}\ntitle: {id} title\nscope: a\ncreated: 2026-09-15\n{extra}---\nbody of {id}\n"
    )
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("legacy");
    for dir in ["a/needs", "a/decisions", "a/gates"] {
        std::fs::create_dir_all(ledger.join(dir)).unwrap();
    }
    std::fs::write(ledger.join("gy.toml"), "[scopes.a]\n").unwrap();
    std::fs::write(
        ledger.join("a/needs/N-1.md"),
        node("N-1", "need", "parent_issue: 6027\n"),
    )
    .unwrap();
    std::fs::write(
        ledger.join("a/decisions/D-1.md"),
        node("D-1", "decision", ""),
    )
    .unwrap();
    std::fs::write(ledger.join("a/gates/G-1.md"), node("G-1", "gate", "")).unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    let data = temp.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    Fixture {
        _temp: temp,
        ledger,
        root,
        data,
    }
}

fn canonical(fx: &Fixture) -> PathBuf {
    gy_ledger::location::dir_in(&fx.data, &fx.root)
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

impl Fixture {
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gy-migrate"));
        command
            .arg(&self.ledger)
            .arg("--root")
            .arg(&self.root)
            .env("XDG_DATA_HOME", &self.data)
            .env("GY_ACTOR", "lead");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run_without_actor(&self, args: &[&str]) -> Output {
        self.command()
            .args(args)
            .env_remove("GY_ACTOR")
            .output()
            .unwrap()
    }
}

#[test]
fn an_existing_canonical_ledger_is_an_error() {
    let fx = fixture();
    gy_ledger::format::write(&canonical(&fx), gy_ledger::FormatVersion::CURRENT).unwrap();
    let out = fx.run(&["--dry-run"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("already exists"), "{}", stderr(&out));
}

#[test]
fn an_actor_is_required_without_dry_run() {
    let fx = fixture();
    let out = fx.run_without_actor(&[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("GY_ACTOR"), "{}", stderr(&out));

    let out = fx.run_without_actor(&["--dry-run"]);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn dry_run_reports_and_writes_nothing() {
    let fx = fixture();
    let out = fx.run(&["--dry-run"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("before: decision 1, gate 1, need 1"),
        "{text}"
    );
    assert!(text.contains("after: decision 1, need 2"), "{text}");
    assert!(text.contains("aliases: 3"), "{text}");
    assert!(text.contains("unrecorded decision scope: 1"), "{text}");
    assert!(text.contains("dropped attributes: parent_issue"), "{text}");
    assert!(text.ends_with('\n'), "the report ends with a newline");
    assert!(!canonical(&fx).exists());
}

#[test]
fn a_real_run_writes_the_ledger() {
    let fx = fixture();
    let out = fx.run(&[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("written: 3 nodes"));
    assert!(canonical(&fx).join("format").is_file());
}
