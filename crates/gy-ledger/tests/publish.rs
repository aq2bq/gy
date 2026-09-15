use gy_ledger::{
    Actor, Alias, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind,
    Publication, Relation, Repository, Store, publish,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn seed<S: Store>(repo: &mut Repository<S>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}

fn render<S: Store>(repo: &Repository<S>) -> Publication {
    publish(repo, None, None, "piko", "/ledger").unwrap()
}

fn render_since<S: Store>(repo: &Repository<S>, since: Option<u64>) -> Publication {
    publish(repo, None, since, "piko", "/ledger").unwrap()
}

/// The section under `heading` in a text.
fn section<'a>(text: &'a str, heading: &str) -> &'a str {
    let start = text.find(heading).unwrap() + heading.len();
    let rest = &text[start..];
    let end = rest.find("\n## ").map_or(rest.len(), |at| at + 1);
    &rest[..end]
}

/// The text of the file whose path contains `needle` in `scope`.
fn file<'a>(publication: &'a Publication, scope: &str, needle: &str) -> &'a str {
    let group = publication
        .scopes
        .iter()
        .find(|group| group.name == scope)
        .unwrap_or_else(|| panic!("no scope {scope}"));
    let file = group
        .files
        .iter()
        .find(|file| file.path.contains(needle))
        .unwrap_or_else(|| panic!("no file {needle} in {scope}"));
    &file.text
}

fn paths(publication: &Publication, scope: &str) -> Vec<String> {
    publication
        .scopes
        .iter()
        .find(|group| group.name == scope)
        .unwrap()
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect()
}

fn decision(hash: &str, alias: &str, title: &str) -> Node {
    let mut node = Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        title,
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap();
    node.add_alias(Alias(alias.into()));
    node
}

#[test]
fn publish_writes_a_scope_and_kind_directory_tree() {
    let mut repo = repo();
    let criterion =
        Node::criterion(id(NodeKind::Criterion, "0001"), SCOPE, DATE, "a criterion").unwrap();
    let decision = decision("0002", "D-1", "a decision");
    let other = Node::need(id(NodeKind::Need, "0003"), "b", DATE, "b need").unwrap();
    seed(&mut repo, &[criterion, decision, other]);

    let publication = render(&repo);
    let names: Vec<&str> = publication
        .scopes
        .iter()
        .map(|group| group.name.as_str())
        .collect();
    assert_eq!(names, ["a", "b"]);
    assert_eq!(
        paths(&publication, "a"),
        [
            "decisions/d-0002-a-decision.md",
            "criteria/ac-0001-a-criterion.md",
            "README.md"
        ]
    );
    assert_eq!(
        paths(&publication, "b"),
        ["needs/n-0003-b-need.md", "README.md"]
    );
}

#[test]
fn a_file_name_is_safe_and_bounded() {
    let mut repo = repo();
    let long = "make / retries: safe, with spaces and a title that runs past the limit";
    let need = Node::need(id(NodeKind::Need, "0004"), SCOPE, DATE, long).unwrap();
    seed(&mut repo, &[need]);

    let publication = render(&repo);
    let path = &paths(&publication, "a")[0];
    let name = path.strip_prefix("needs/n-0004-").unwrap();
    let name = name.strip_suffix(".md").unwrap();
    assert!(!name.contains('/'), "{path}");
    assert!(!name.contains([':', ',', ' ']), "{path}");
    assert!(name.chars().count() <= 40, "{path}");
    assert_eq!(
        path,
        "needs/n-0004-make-retries-safe-with-spaces-and-a-titl.md"
    );
}

#[test]
fn a_decision_file_puts_its_lineage_first() {
    let mut repo = repo();
    let old = decision("0005", "D-1", "the old decision");
    let mut new = decision("0006", "D-2", "the new decision");
    new.link(
        Link::new(new.id().clone(), Relation::Narrows, old.id().clone())
            .unwrap()
            .with_mark(Some("the changed part".into())),
    );
    seed(&mut repo, &[old, new]);

    let publication = render(&repo);
    let newer = file(&publication, "a", "d-0006");
    assert!(
        newer.starts_with("# d-0006 (D-2) the new decision\n"),
        "{newer}"
    );
    assert!(
        newer
            .contains("## 関係\n- narrows d-0005 (D-1) the old decision（mark: the changed part）"),
        "{newer}"
    );
    let older = file(&publication, "a", "d-0005");
    assert!(
        older.contains("## 関係\n- narrowed-by d-0006 (D-2) the new decision"),
        "{older}"
    );
}

#[test]
fn an_index_lists_nodes_with_links_and_sections() {
    let mut repo = repo();
    let criterion = Node::criterion(id(NodeKind::Criterion, "0001"), SCOPE, DATE, "an AC").unwrap();
    let decision = decision("0002", "D-1", "a decision");
    seed(&mut repo, &[criterion, decision]);

    let publication = render(&repo);
    let index = file(&publication, "a", "README.md");
    assert!(index.starts_with("# gy の公開物 — a\n"), "{index}");
    for marker in [
        "- seq: ",
        "- scope: a",
        "- 書き手: piko",
        "- 正本: /ledger",
        "## 読み方",
        "## 一覧",
        "## 履歴",
        "## 診断",
    ] {
        assert!(index.contains(marker), "missing {marker}:\n{index}");
    }
    assert!(
        index.contains("- [ac-0001 an AC](criteria/ac-0001-an-AC.md) — unsatisfied"),
        "{index}"
    );
    assert!(
        index.contains("- [d-0002 (D-1) a decision](decisions/d-0002-a-decision.md)"),
        "{index}"
    );
}

#[test]
fn the_index_history_honors_since() {
    let mut repo = repo();
    seed(
        &mut repo,
        &[Node::need(id(NodeKind::Need, "0001"), SCOPE, DATE, "first need").unwrap()],
    );
    seed(
        &mut repo,
        &[Node::need(id(NodeKind::Need, "0002"), SCOPE, DATE, "second need").unwrap()],
    );

    let index = render_since(&repo, Some(1));
    let index = file(&index, "a", "README.md");
    let history = section(index, "## 履歴");
    assert!(history.contains("second need"), "{history}");
    assert!(!history.contains("first need"), "{history}");
}

#[test]
fn two_runs_produce_the_same_files() {
    let mut repo = repo();
    let criterion = Node::criterion(id(NodeKind::Criterion, "0001"), SCOPE, DATE, "an AC").unwrap();
    let decision = decision("0002", "D-1", "a decision");
    seed(&mut repo, &[criterion, decision]);

    let first = render(&repo);
    let second = render(&repo);
    assert_eq!(paths(&first, "a"), paths(&second, "a"));
    let strip = |text: &str| {
        text.lines()
            .filter(|line| !line.starts_with("- 生成:"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    for group in &first.scopes {
        for entry in &group.files {
            assert_eq!(
                strip(&entry.text),
                strip(file(&second, &group.name, &entry.path)),
                "{}",
                entry.path
            );
        }
    }
}
