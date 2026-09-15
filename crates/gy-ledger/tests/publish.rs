use gy_ledger::{
    Actor, Alias, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Ref, Repository,
    RequirementState, Store, publish,
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
