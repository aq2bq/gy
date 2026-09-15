use gy_ledger::{
    Actor, Alias, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeData, NodeId,
    NodeKind, Ref, Relation, Repository, RequirementState, Store, describe, publish,
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

fn question(hash: &str, alias: &str, decider: &str, title: &str) -> Node {
    let mut node = Node::question(id(NodeKind::Question, hash), SCOPE, DATE, title).unwrap();
    if let NodeData::Question(data) = node.data_mut() {
        data.decider = Some(decider.to_string());
        data.options = vec!["a".into(), "b".into()];
    }
    node.add_alias(Alias(alias.into()));
    node
}

fn ledger() -> Repository<MemoryStore> {
    let mut repo = repo();
    let mut master = question("0001", "Q-1", "master", "master が決める論点");
    master.set_body("## 推奨\n\nA を選ぶ。");
    let other = question("0002", "Q-2", "lead", "lead が決める論点");
    let mut requirement = Node::requirement(
        id(NodeKind::Requirement, "0003"),
        SCOPE,
        DATE,
        "確定待ちの要求",
        RequirementState::Filed,
    )
    .unwrap();
    requirement.add_alias(Alias("R-1".into()));
    if let NodeData::Requirement(data) = requirement.data_mut() {
        data.reference = Some(Ref("https://example/1".into()));
    }
    requirement.set_free("next_evidence", "設計の確認");
    requirement.set_free("responsible", "master");
    let outside = Node::need(id(NodeKind::Need, "0004"), "b", DATE, "別スコープ").unwrap();
    seed(&mut repo, &[master, other, requirement, outside]);
    repo
}

#[test]
fn publish_answers_the_master_questions() {
    let repo = ledger();
    let text = publish(&repo, None, None).unwrap();

    assert!(text.contains("## いま判断待ち"), "{text}");
    assert!(text.contains("## 未決の論点"), "{text}");
    assert!(text.contains("## 注意"), "{text}");
    assert!(!text.contains("## ノード"), "{text}");

    let planned: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("- 論点:"))
        .collect();
    assert_eq!(planned.len(), 1, "{text}");
    assert!(planned[0].contains("q-0001"), "{text}");
    assert!(planned[0].contains("(Q-1)"), "{text}");
    assert!(planned[0].contains("master が決める論点"), "{text}");
    assert!(text.contains("推奨: A を選ぶ。"), "{text}");

    assert!(text.contains("### master"), "{text}");
    assert!(text.contains("### lead"), "{text}");
    assert!(text.contains("lead が決める論点"), "{text}");

    let filed = text
        .lines()
        .find(|line| line.starts_with("- 要求:"))
        .unwrap();
    assert!(filed.contains("r-0003"), "{text}");
    assert!(filed.contains("(R-1, https://example/1)"), "{text}");
    assert!(filed.contains("確定待ちの要求"), "{text}");
    assert!(text.contains("next_evidence: 設計の確認"), "{text}");
    assert!(text.contains("responsible: master"), "{text}");
}

#[test]
fn publish_keeps_ids_with_titles_and_filters_by_scope() {
    let repo = ledger();
    let text = publish(&repo, Some("a"), None).unwrap();
    assert!(!text.contains("別スコープ"), "{text}");
    assert!(!text.lines().any(|line| line.trim() == "q-0001"), "{text}");
    assert!(!text.lines().any(|line| line.trim() == "r-0003"), "{text}");

    let scoped = publish(&repo, Some("b"), None).unwrap();
    assert!(!scoped.contains("master が決める論点"), "{scoped}");
    assert!(scoped.contains("いま判断待ちは無し。"), "{scoped}");
}

fn decision(hash: &str, alias: &str, title: &str, body: &str) -> Node {
    let mut node = Node::decision(
        id(NodeKind::Decision, hash),
        SCOPE,
        DATE,
        title,
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap();
    node.set_body(body);
    node.add_alias(Alias(alias.into()));
    node
}

fn lineage_ledger() -> Repository<MemoryStore> {
    let mut repo = repo();
    let old = decision("0010", "D-1", "古い決定", "the changed part");
    let mut new = decision("0011", "D-2", "新しい決定", "");
    new.link(
        Link::new(new.id().clone(), Relation::Narrows, old.id().clone())
            .unwrap()
            .with_mark(Some("the changed part".into())),
    );
    let mut need = Node::need(id(NodeKind::Need, "0012"), SCOPE, DATE, "元のニーズ").unwrap();
    need.add_alias(Alias("N-1".into()));
    let mut requirement = Node::requirement(
        id(NodeKind::Requirement, "0013"),
        SCOPE,
        DATE,
        "要求",
        RequirementState::Filed,
    )
    .unwrap();
    requirement.add_alias(Alias("R-1".into()));
    if let NodeData::Requirement(data) = requirement.data_mut() {
        data.reference = Some(Ref("https://example/9".into()));
    }
    need.link(
        Link::new(
            need.id().clone(),
            Relation::FiledAs,
            requirement.id().clone(),
        )
        .unwrap(),
    );
    requirement.link(
        Link::new(
            requirement.id().clone(),
            Relation::ReliesOn,
            old.id().clone(),
        )
        .unwrap(),
    );
    seed(&mut repo, &[old, need, requirement]);
    repo.transaction("decide d-0011", "test", |repo| repo.put(&new))
        .unwrap();
    repo
}

#[test]
fn describe_shows_only_the_named_nodes() {
    let repo = lineage_ledger();
    let ids = [
        id(NodeKind::Decision, "0011"),
        id(NodeKind::Requirement, "0013"),
    ];
    let text = describe(&repo, &ids).unwrap();
    assert!(
        !text
            .lines()
            .any(|line| line.starts_with("## ") && !line.starts_with("### ")),
        "{text}"
    );
    let headings = text.lines().filter(|line| line.starts_with("### ")).count();
    assert_eq!(headings, 2, "{text}");
    assert!(text.contains("狭める"), "{text}");
    assert!(text.contains("mark: the changed part"), "{text}");
    assert!(text.contains("来歴:"), "{text}");
    assert!(text.contains("- ニーズ: "), "{text}");
    assert!(text.contains("- 依拠する決定: "), "{text}");
    assert!(text.contains("- 状態: filed"), "{text}");
}

#[test]
fn attention_counts_without_listing() {
    let mut repo = repo();
    let mut need = Node::need(id(NodeKind::Need, "0020"), SCOPE, DATE, "dangling").unwrap();
    let missing = id(NodeKind::Criterion, "9999");
    need.link(Link::new(need.id().clone(), Relation::Targets, missing).unwrap());
    seed(&mut repo, &[need]);

    let text = publish(&repo, None, None).unwrap();
    assert!(text.contains("## 注意"), "{text}");
    assert!(text.contains("参照先の無い辺: 1"), "{text}");
    assert!(!text.contains("missing"), "{text}");
}

#[test]
fn publish_lists_the_period_changes() {
    let repo = lineage_ledger();
    let text = publish(&repo, None, Some(0)).unwrap();
    assert!(text.contains("## この期間の変更"), "{text}");
    assert!(text.contains("【決定の作成】"), "{text}");
    assert!(text.contains("d-0011"), "{text}");
}
