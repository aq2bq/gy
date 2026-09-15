use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    _temp: tempfile::TempDir,
    ledger: PathBuf,
    root: PathBuf,
    data: PathBuf,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("legacy");
    std::fs::create_dir_all(ledger.join("a/criteria")).unwrap();
    std::fs::write(ledger.join("gy.toml"), "[scopes.a]\n").unwrap();
    std::fs::write(
        ledger.join("a/criteria/AC-1.md"),
        "---\nid: 'AC-1'\ntype: criterion\ntitle: 'a criterion'\nscope: a\ncreated: 2026-09-15\n---\nbody\n",
    )
    .unwrap();
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

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

impl Fixture {
    fn run(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_gy-migrate"))
            .arg(&self.ledger)
            .arg("--root")
            .arg(&self.root)
            .env("XDG_DATA_HOME", &self.data)
            .env("GY_ACTOR", "lead")
            .output()
            .unwrap()
    }

    fn gy_toml(&self) -> String {
        std::fs::read_to_string(self.root.join("gy.toml")).unwrap()
    }
}

#[test]
fn a_missing_gy_toml_is_generated_with_the_scopes() {
    let fx = fixture();
    let out = fx.run();
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(fx.gy_toml().contains("[scopes.a]"), "{}", fx.gy_toml());
}

#[test]
fn an_existing_gy_toml_with_the_scope_is_kept() {
    let fx = fixture();
    std::fs::write(fx.root.join("gy.toml"), "# mine\n[scopes.a]\n").unwrap();
    let out = fx.run();
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(fx.gy_toml().contains("# mine"));
}

#[test]
fn an_existing_gy_toml_missing_a_scope_is_an_error() {
    let fx = fixture();
    std::fs::write(fx.root.join("gy.toml"), "[scopes.b]\n").unwrap();
    let out = fx.run();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("does not name scopes"),
        "{}",
        stderr(&out)
    );
}
