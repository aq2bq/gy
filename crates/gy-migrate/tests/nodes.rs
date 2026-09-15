use gy_ledger::{FileStore, Node, NodeData, Ref, Repository, RequirementState, Store, log};
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    _temp: tempfile::TempDir,
    ledger: PathBuf,
    root: PathBuf,
    data: PathBuf,
}

fn md(id: &str, kind: &str, title: &str, extra: &str) -> String {
    format!(
        "---\nid: '{id}'\ntype: {kind}\ntitle: {title}\nscope: a\ncreated: 2026-09-15\n{extra}---\nbody of {id}\n"
    )
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("legacy");
    for dir in [
        "a/needs",
        "a/questions",
        "a/decisions",
        "a/requirements",
        "a/criteria",
        "a/gates",
    ] {
        std::fs::create_dir_all(ledger.join(dir)).unwrap();
    }
    std::fs::write(ledger.join("gy.toml"), "[scopes.a]\n").unwrap();
    let files = [
        (
            "a/needs/N-1.md",
            md(
                "N-1",
                "need",
                "need one",
                "next_evidence: watch\nresponsible: master\nremaining_work: half\n",
            ),
        ),
        (
            "a/needs/N-2.md",
            md(
                "N-2",
                "need",
                "need two",
                "status: complete\ndropped_reason: no longer needed\ndropped_by: master\n",
            ),
        ),
        (
            "a/needs/N-3.md",
            md("N-3", "need", "need three", "filed-as:\n- '#12'\n"),
        ),
        (
            "a/decisions/D-2.md",
            md(
                "D-2",
                "decision",
                "decision two",
                "decision_scope: applies at dawn\n",
            ),
        ),
        (
            "a/decisions/D-3.md",
            md("D-3", "decision", "decision three", ""),
        ),
        (
            "a/questions/Q-4.md",
            md(
                "Q-4",
                "question",
                "question four",
                "decider: master\noptions:\n- a\n- b\nstatus: closed\nclosed_by: fact\nclosure_note: observed\n",
            ),
        ),
        (
            "a/criteria/AC-5.md",
            md(
                "AC-5",
                "criterion",
                "criterion five",
                "satisfied: true\nsatisfied_at: 2026-09-15\nevidence: verified\n",
            ),
        ),
        (
            "a/requirements/#12.md",
            md(
                "#12",
                "requirement",
                "requirement twelve",
                "status: complete\nnext_evidence: x\nresponsible: master\n",
            ),
        ),
        ("a/gates/G-1.md", md("G-1", "gate", "gate one", "")),
    ];
    for (path, content) in files {
        std::fs::write(ledger.join(path), content).unwrap();
    }
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
    fn canonical(&self) -> PathBuf {
        gy_ledger::location::dir_in(&self.data, &self.root)
    }

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
        let store = FileStore::open_with(&self.canonical(), |_| Some("lead".into())).unwrap();
        Repository::new(store)
    }
}

fn node<S: Store>(repository: &Repository<S>, alias: &str) -> Node {
    let id = repository.resolve(alias).unwrap();
    repository.get(&id).unwrap().unwrap()
}

#[test]
fn nodes_are_transferred_with_aliases_and_refs() {
    let fx = fixture();
    let out = fx.run(&["--ref-base", "https://example/issues/"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let repository = fx.open();

    let need = node(&repository, "N-1");
    assert_eq!(need.kind().name(), "need");
    assert_eq!(need.free("responsible").map(String::as_str), Some("master"));
    assert_eq!(
        need.free("next_evidence").map(String::as_str),
        Some("watch")
    );

    let gate = node(&repository, "G-1");
    assert_eq!(gate.kind().name(), "need");
    assert!(gate.body().starts_with("gate から移行"), "{}", gate.body());

    let closed = node(&repository, "N-2");
    match closed.data() {
        NodeData::Need(data) => {
            let closed = data.closed.as_ref().unwrap();
            assert_eq!(closed.evidence, "no longer needed (master)");
        }
        _ => panic!("not a need"),
    }

    let derived = node(&repository, "N-3");
    match derived.data() {
        NodeData::Need(data) => assert!(data.closed.is_none()),
        _ => panic!("not a need"),
    }

    let unrecorded = node(&repository, "D-3");
    match unrecorded.data() {
        NodeData::Decision(data) => assert!(data.scope.is_unrecorded()),
        _ => panic!("not a decision"),
    }

    let requirement = node(&repository, "#12");
    match requirement.data() {
        NodeData::Requirement(data) => {
            assert_eq!(data.state, RequirementState::Done);
            assert_eq!(
                data.reference,
                Some(Ref("https://example/issues/12".into()))
            );
        }
        _ => panic!("not a requirement"),
    }

    let question = node(&repository, "Q-4");
    match question.data() {
        NodeData::Question(data) => {
            assert_eq!(data.decider.as_deref(), Some("master"));
            assert_eq!(data.options.len(), 2);
            assert_eq!(data.evidence.as_deref(), Some("observed"));
            assert!(data.closure.is_some());
        }
        _ => panic!("not a question"),
    }

    let criterion = node(&repository, "AC-5");
    match criterion.data() {
        NodeData::Criterion(data) => {
            assert!(data.satisfied);
            assert_eq!(data.evidence.as_deref(), Some("verified"));
        }
        _ => panic!("not a criterion"),
    }
}

#[test]
fn the_transfer_is_one_transaction() {
    let fx = fixture();
    let out = fx.run(&["--ref-base", "https://example/issues/"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let (events, _) = log::read(&fx.canonical()).unwrap();
    assert_eq!(events.len(), 1);
}
