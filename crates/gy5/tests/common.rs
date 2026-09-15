#![allow(dead_code)]
//! Shared helpers: a repository root with a gy.toml, its XDG data directory,
//! and running the gy5 binary against them.
use gy_ledger::{FileStore, FormatVersion, Node, Repository, format, location};
use std::path::PathBuf;
use std::process::{Command, Output};

pub struct Fixture {
    _temp: tempfile::TempDir,
    pub root: PathBuf,
    pub data: PathBuf,
}

pub fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("gy.toml"), "[scopes.a]\n").unwrap();
    let data = temp.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    Fixture {
        _temp: temp,
        root,
        data,
    }
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

pub fn first_line(output: &Output) -> String {
    stdout(output)
        .lines()
        .next()
        .unwrap_or_default()
        .to_string()
}

impl Fixture {
    pub fn ledger(&self) -> PathBuf {
        location::dir_in(&self.data, &self.root)
    }

    /// Create the ledger and write nodes, as an earlier write would have.
    pub fn seed(&self, nodes: &[Node]) {
        let ledger = self.ledger();
        format::write(&ledger, FormatVersion::CURRENT).unwrap();
        let store = FileStore::open_with(&ledger, |_| Some("piko".into())).unwrap();
        let mut repository = Repository::new(store);
        repository
            .transaction("seed", "test", |repository| {
                for node in nodes {
                    repository.put(node)?;
                }
                Ok(())
            })
            .unwrap();
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    pub fn run_without_actor(&self, args: &[&str]) -> Output {
        self.command(args).env_remove("GY_ACTOR").output().unwrap()
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gy5"));
        command
            .current_dir(&self.root)
            .env("XDG_DATA_HOME", &self.data)
            .env("GY_ACTOR", "piko")
            .args(args);
        command
    }
}
