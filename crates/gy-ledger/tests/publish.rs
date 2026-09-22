//! publish as a Markdown wiki (n-b6c9, ac-9bc1): a page per need and per
//! decision, the nodes they reach in full, sharing, and `loose.md`.
use gy_ledger::{
    Actor, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind, Publication,
    Relation, Repository, RequirementState, Store, publish,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

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

fn link(from: &Node, relation: Relation, to: &Node) -> Link {
    Link::new(from.id().clone(), relation, to.id().clone()).unwrap()
}

fn criterion(hash: &str, title: &str) -> Node {
    Node::criterion(id(NodeKind::Criterion, hash), SCOPE, DATE, title).unwrap()
}

fn question(hash: &str, title: &str) -> Node {
    Node::question(id(NodeKind::Question, hash), SCOPE, DATE, title).unwrap()
}

fn requirement(hash: &str, title: &str) -> Node {
    Node::requirement(
        id(NodeKind::Requirement, hash),
        SCOPE,
        DATE,
        title,
        RequirementState::Filed,
    )
    .unwrap()
}

fn decision(hash: &str, title: &str) -> Node {
    Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        title,
        DecisionScope::recorded("applies at dawn").unwrap(),
    )
    .unwrap()
}

/// One need with a criterion, a requirement that raised a question, and the
/// decision that spawned it; plus an unbuilt criterion nothing reaches.
fn wiki() -> Publication {
    let mut repo = repo();
    let built = criterion("0001", "a criterion");
    let loose = criterion("0002", "an unbuilt criterion");
    let mut need = Node::need(id(NodeKind::Need, "0003"), SCOPE, DATE, "a need").unwrap();
    let mut requirement = requirement("0004", "a requirement");
    let raised = question("0005", "a raised question");
    let decision = decision("0006", "a decision");
    need.link(link(&need, Relation::Targets, &built));
    need.link(link(&need, Relation::FiledAs, &requirement));
    need.link(link(&need, Relation::SpawnedBy, &decision));
    requirement.link(link(&requirement, Relation::Raised, &raised));
    seed(
        &mut repo,
        &[built, loose, need, requirement, raised, decision],
    );
    publish(&repo, Some(SCOPE)).unwrap()
}

fn paths(publication: &Publication) -> Vec<String> {
    let mut names: Vec<String> = publication
        .scopes
        .iter()
        .flat_map(|scope| scope.files.iter().map(|file| file.path.clone()))
        .collect();
    names.sort();
    names
}

fn file<'a>(publication: &'a Publication, path: &str) -> &'a str {
    publication
        .scopes
        .iter()
        .flat_map(|scope| &scope.files)
        .find(|file| file.path == path)
        .unwrap_or_else(|| panic!("no file {path}"))
        .text
        .as_str()
}

#[test]
fn only_needs_and_decisions_get_pages() {
    let publication = wiki();
    assert_eq!(
        paths(&publication),
        ["d-0006.md", "loose.md", "n-0003.md"],
        "no per-kind directories, one page per vertex"
    );
}

#[test]
fn every_node_of_the_scope_is_readable_in_full() {
    let publication = wiki();
    let text: String = publication
        .scopes
        .iter()
        .flat_map(|scope| &scope.files)
        .map(|file| file.text.as_str())
        .collect();
    for id in ["ac-0001", "ac-0002", "n-0003", "r-0004", "q-0005", "d-0006"] {
        assert!(text.contains(id), "{id} is not shown anywhere");
    }
}

#[test]
fn a_need_page_carries_front_matter_and_expands_what_it_reaches() {
    let publication = wiki();
    let page = file(&publication, "n-0003.md");
    assert!(
        page.starts_with(
            "---\nid: n-0003\nkind: need\nstate: open\nscope: a\ncreated: 2026-09-15\n\
             targets: [ac-0001]\nfiled_as: [r-0004]\nspawned_by: [d-0006]\n---\n\n# a need\n\n"
        ),
        "{page}"
    );
    assert!(page.contains("## What it comes from"), "{page}");
    assert!(page.contains("## What must hold"), "{page}");
    assert!(
        page.contains("#### <a id=\"ac0001\"></a>Criterion `ac-0001` — a criterion"),
        "{page}"
    );
    assert!(page.contains("- Not met yet"), "{page}");
    assert!(page.contains("## What was built"), "{page}");
    assert!(
        page.contains("#### <a id=\"r0004\"></a>Requirement `r-0004` — a requirement"),
        "{page}"
    );
    assert!(page.contains("- State: **filed**"), "{page}");
    assert!(page.contains("## What it raised"), "{page}");
    assert!(
        page.contains("#### <a id=\"q0005\"></a>Question `q-0005` — a raised question"),
        "{page}"
    );
}

#[test]
fn a_decision_page_carries_its_scope_and_lineage() {
    let publication = wiki();
    let page = file(&publication, "d-0006.md");
    assert!(page.contains("## Where it holds"), "{page}");
    assert!(page.contains("applies at dawn"), "{page}");
    assert!(
        page.contains("## What came out of it") && page.contains("The need [`n-0003`"),
        "{page}"
    );
}

#[test]
fn two_vertices_sharing_a_node_both_show_it_and_say_so() {
    let mut repo = repo();
    let shared = criterion("0001", "a shared criterion");
    let mut first = Node::need(id(NodeKind::Need, "0002"), SCOPE, DATE, "first").unwrap();
    let mut second = Node::need(id(NodeKind::Need, "0003"), SCOPE, DATE, "second").unwrap();
    first.link(link(&first, Relation::Targets, &shared));
    second.link(link(&second, Relation::Targets, &shared));
    seed(&mut repo, &[shared, first, second]);

    let publication = publish(&repo, Some(SCOPE)).unwrap();
    let first_page = file(&publication, "n-0002.md");
    let second_page = file(&publication, "n-0003.md");
    assert!(first_page.contains("ac-0001"), "{first_page}");
    assert!(second_page.contains("ac-0001"), "{second_page}");
    assert!(
        first_page.contains("- Shared with: [`n-0003` second](n-0003.md)"),
        "{first_page}"
    );
    assert!(
        second_page.contains("- Shared with: [`n-0002` first](n-0002.md)"),
        "{second_page}"
    );
}

#[test]
fn a_node_no_page_reaches_goes_to_loose() {
    let publication = wiki();
    let loose = file(&publication, "loose.md");
    assert!(loose.starts_with("# On their own\n"), "{loose}");
    assert!(loose.contains("ac-0002"), "{loose}");
    assert!(loose.contains("an unbuilt criterion"), "{loose}");
}

#[test]
fn a_narrowed_passage_is_marked_for_markdown() {
    let mut repo = repo();
    let old = Node::decision(
        id(NodeKind::Decision, "0007"),
        SCOPE,
        DATE,
        "the old decision",
        DecisionScope::recorded("the read scope and an expiry").unwrap(),
    )
    .unwrap();
    let mut new = decision("0008", "the new decision");
    new.link(
        link(&new, Relation::Narrows, &old).with_mark(Some("the read scope and an expiry".into())),
    );
    seed(&mut repo, &[old, new]);

    let publication = publish(&repo, Some(SCOPE)).unwrap();
    let page = file(&publication, "d-0007.md");
    assert!(
        page.contains("~~the read scope and an expiry~~ *(retracted by d-0008)*"),
        "{page}"
    );
    assert!(page.contains("- Narrowed by"), "{page}");
}

#[test]
fn another_scope_is_plain_text_not_a_link() {
    let mut repo = repo();
    let elsewhere = Node::need(id(NodeKind::Need, "0001"), "b", DATE, "a need in b").unwrap();
    let mut need = Node::need(id(NodeKind::Need, "0002"), SCOPE, DATE, "a need").unwrap();
    need.link(link(&need, Relation::DependsOn, &elsewhere));
    seed(&mut repo, &[elsewhere, need]);

    let publication = publish(&repo, Some(SCOPE)).unwrap();
    assert_eq!(paths(&publication), ["n-0002.md"]);
    let page = file(&publication, "n-0002.md");
    assert!(page.contains("*(in b)*"), "{page}");
    assert!(!page.contains("](n-0001.md)"), "{page}");
}

#[test]
fn two_runs_are_byte_identical() {
    let first = wiki();
    let second = wiki();
    assert_eq!(paths(&first), paths(&second));
    for path in paths(&first) {
        assert_eq!(file(&first, &path), file(&second, &path), "{path}");
    }
}
