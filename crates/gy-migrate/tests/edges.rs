use gy_ledger::{FileStore, Node, Relation, Repository, Store};
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    _temp: tempfile::TempDir,
    ledger: PathBuf,
    root: PathBuf,
    data: PathBuf,
}

fn md(id: &str, kind: &str, title: &str, extra: &str, body: &str) -> String {
    format!(
        "---\nid: '{id}'\ntype: {kind}\ntitle: {title}\nscope: a\ncreated: 2026-09-15\n{extra}---\n{body}\n"
    )
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let ledger = temp.path().join("legacy");
    for dir in [
        "a/needs",
        "a/questions",
        "a/decisions",
        "a/criteria",
        "a/requirements",
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
                "targets:\n- AC-2\ndepends-on:\n- N-3\nwaiting-on:\n- Q-4\n- '#9'\n",
                "body of N-1",
            ),
        ),
        (
            "a/needs/N-3.md",
            md("N-3", "need", "need three", "", "body of N-3"),
        ),
        (
            "a/needs/N-7.md",
            md(
                "N-7",
                "need",
                "need seven",
                "targets:\n- AC-999\n",
                "body of N-7",
            ),
        ),
        (
            "a/needs/N-10.md",
            md(
                "N-10",
                "need",
                "need ten",
                "narrows:\n- id: D-5\n  mark: old text\n",
                "body of N-10",
            ),
        ),
        (
            "a/questions/Q-4.md",
            md(
                "Q-4",
                "question",
                "question four",
                "decider: master\noptions:\n- a\n- b\ncloses:\n- D-5\nbelongs-to:\n- N-3\n- '#9'\n",
                "body of Q-4",
            ),
        ),
        (
            "a/decisions/D-5.md",
            md(
                "D-5",
                "decision",
                "decision five",
                "closes:\n- Q-4\n",
                "the old text here",
            ),
        ),
        (
            "a/decisions/D-6.md",
            md(
                "D-6",
                "decision",
                "decision six",
                "narrows:\n- id: D-5\n  mark: old text\n",
                "body of D-6",
            ),
        ),
        (
            "a/requirements/9.md",
            md("#9", "requirement", "requirement nine", "", "body of R-9"),
        ),
        (
            "a/criteria/AC-2.md",
            md("AC-2", "criterion", "criterion two", "", "body of AC-2"),
        ),
        (
            "a/gates/G-8.md",
            md(
                "G-8",
                "gate",
                "gate eight",
                "measured-by:\n- Q-4\n",
                "body of G-8",
            ),
        ),
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

fn edge_target<S: Store>(
    repository: &Repository<S>,
    from: &Node,
    relation: Relation,
    alias: &str,
) -> bool {
    let to = repository.resolve(alias).unwrap();
    from.links()
        .iter()
        .any(|edge| edge.label == relation && edge.to == to)
}

#[test]
fn edges_marks_and_waits_on_are_written_and_the_rest_reported() {
    let fx = fixture();
    let out = fx.run(&["--ref-base", "https://example/"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("edges written: 4"), "{text}");
    assert!(text.contains("edges missing a target: 1"), "{text}");
    assert!(text.contains("edges not allowed by kind: 1"), "{text}");
    assert!(text.contains("waits-on written: 4"), "{text}");
    assert!(text.contains("waits-on skipped: 0"), "{text}");
    assert!(
        text.contains("belongs-to kept as free attributes: 1"),
        "{text}"
    );
    assert!(text.contains("missing: N-7 targets AC-999"), "{text}");
    assert!(text.contains("not allowed: N-10 narrows D-5"), "{text}");

    let repository = fx.open();
    let need = node(&repository, "N-1");
    assert!(edge_target(&repository, &need, Relation::Targets, "AC-2"));
    assert!(edge_target(&repository, &need, Relation::DependsOn, "N-3"));
    assert!(edge_target(&repository, &need, Relation::WaitsOn, "Q-4"));
    assert!(edge_target(&repository, &need, Relation::WaitsOn, "#9"));
    assert_eq!(need.free("waiting-on"), None);

    // A question's belonging to a need means the need waits on the question;
    // a target that is not a need stays as a free attribute.
    let three = node(&repository, "N-3");
    assert!(edge_target(&repository, &three, Relation::WaitsOn, "Q-4"));
    let q4 = node(&repository, "Q-4");
    assert_eq!(q4.free("belongs-to").map(String::as_str), Some("#9"));

    let gate = node(&repository, "G-8");
    assert!(edge_target(&repository, &gate, Relation::WaitsOn, "Q-4"));

    let decision = node(&repository, "D-6");
    let narrows = decision
        .links()
        .iter()
        .find(|edge| edge.label == Relation::Narrows)
        .unwrap();
    assert_eq!(narrows.mark.as_deref(), Some("old text"));

    // 0.4 stores closes on both sides; only the question side is an edge.
    let question = node(&repository, "Q-4");
    assert!(edge_target(&repository, &question, Relation::Closes, "D-5"));
    let reversed = node(&repository, "D-5");
    assert!(
        !reversed
            .links()
            .iter()
            .any(|edge| edge.label == Relation::Closes)
    );
}
