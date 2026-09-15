use gy_ledger::{FileStore, Node, NodeData, Repository, RequirementState, Store};
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    _temp: tempfile::TempDir,
    ledger: PathBuf,
    root: PathBuf,
    data: PathBuf,
    publication: PathBuf,
}

fn md(id: &str, extra: &str) -> String {
    format!(
        "---\nid: '{id}'\ntype: requirement\ntitle: '{id} title'\nscope: a\ncreated: 2026-09-15\n{extra}---\nbody of {id}\n"
    )
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("legacy");
    std::fs::create_dir_all(ledger.join("a/requirements")).unwrap();
    std::fs::write(ledger.join("gy.toml"), "[scopes.a]\n").unwrap();
    let transition = "transitions:\n- from: awaiting-approval\n  to: awaiting-implementation\n  evidence: design approved\n  at: '2026-09-10'\n";
    let files = [
        ("#1.md", md("#1", "status: unfiled\n")),
        ("#2.md", md("#2", "status: defining\n")),
        ("#3.md", md("#3", "status: awaiting-design\n")),
        ("#4.md", md("#4", "status: awaiting-approval\n")),
        (
            "#5.md",
            md(
                "#5",
                &format!("status: awaiting-implementation\n{transition}"),
            ),
        ),
        ("#6.md", md("#6", "status: awaiting-cleanup\n")),
        (
            "#7.md",
            md(
                "#7",
                "status: complete\npr_url: https://pr/1\ntransitions:\n- to: complete\n  at: '2026-09-12'\n",
            ),
        ),
        ("#8.md", md("#8", "status: reported\n")),
    ];
    for (name, content) in files {
        std::fs::write(ledger.join("a/requirements").join(name), content).unwrap();
    }
    let root = temp.path().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    let data = temp.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let publication = temp.path().join("pub");
    Fixture {
        _temp: temp,
        ledger,
        root,
        data,
        publication,
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

impl Fixture {
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_gy-migrate"))
            .arg(&self.ledger)
            .arg("--root")
            .arg(&self.root)
            .env("XDG_DATA_HOME", &self.data)
            .env("GY_ACTOR", "lead")
            .args(args)
            .output()
            .unwrap()
    }

    fn open(&self) -> Repository<FileStore> {
        let ledger = gy_ledger::location::dir_in(&self.data, &self.root);
        Repository::new(FileStore::open_with(&ledger, |_| Some("lead".into())).unwrap())
    }
}

fn node<S: Store>(repository: &Repository<S>, alias: &str) -> Node {
    let id = repository.resolve(alias).unwrap();
    repository.get(&id).unwrap().unwrap()
}

fn requirement<S: Store>(
    repository: &Repository<S>,
    alias: &str,
) -> (RequirementState, gy_ledger::Requirement) {
    match node(repository, alias).data().clone() {
        NodeData::Requirement(data) => (data.state, data),
        _ => panic!("not a requirement"),
    }
}

#[test]
fn requirement_states_and_records_are_mapped() {
    let fx = fixture();
    let out = fx.run(&["--ref-base", "https://example/"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("unmapped requirement status: 1"), "{text}");
    assert!(text.contains("#8 reported"), "{text}");

    let repository = fx.open();
    for id in ["#1", "#2", "#3", "#4", "#8"] {
        assert_eq!(
            requirement(&repository, id).0,
            RequirementState::Filed,
            "{id}"
        );
    }
    let (state, approved) = requirement(&repository, "#5");
    assert_eq!(state, RequirementState::Approved);
    let approval = approved.approval.as_ref().unwrap();
    assert_eq!(approval.design, "design approved");
    assert_eq!(approval.evidence, "design approved");
    assert_eq!(approval.heard_by, "migrated");
    assert_eq!(approval.at, "2026-09-10");

    let (_, migrated) = requirement(&repository, "#6");
    assert_eq!(migrated.approval.as_ref().unwrap().design, "migrated");

    let (state, done) = requirement(&repository, "#7");
    assert_eq!(state, RequirementState::Done);
    let completion = done.completion.as_ref().unwrap();
    assert_eq!(completion.evidence, "https://pr/1");
    assert_eq!(completion.at, "2026-09-12");
}

#[test]
fn records_are_frozen_and_the_report_is_written() {
    let fx = fixture();
    let publication = fx.publication.display().to_string();
    let out = fx.run(&[
        "--ref-base",
        "https://example/",
        "--publication",
        &publication,
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("frozen records: 2"), "{text}");
    assert!(text.contains("requirement states:"), "{text}");
    assert!(text.contains("#8 reported -> filed"), "{text}");

    let frozen = std::fs::read_to_string(fx.publication.join("a/#5.md")).unwrap();
    assert!(frozen.contains("transitions:"), "{frozen}");
    let report = std::fs::read_to_string(fx.publication.join("migration-report.md")).unwrap();
    assert!(report.contains("frozen records: 2"), "{report}");

    let node = node(&fx.open(), "#5");
    assert_eq!(
        node.free("legacy_records").map(String::as_str),
        Some("a/#5.md")
    );
}
