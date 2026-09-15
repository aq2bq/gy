use gy_ledger::{
    Actor, Alias, Approval, Closure, DecisionScope, FormatVersion, Link, MemoryStore, Node,
    NodeData, NodeId, NodeKind, Ref, Relation, Repository, RequirementState, Store, publish,
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

fn render<S: Store>(repo: &Repository<S>, since: Option<u64>) -> String {
    publish(repo, None, since, "piko", "/ledger").unwrap()
}

/// A decision, a closed question, a satisfied criterion, an approved
/// requirement with a reference, and a need filed as it.
fn ledger() -> Repository<MemoryStore> {
    let mut repo = repo();
    let mut decision = Node::decision(
        id(NodeKind::Decision, "0002"),
        SCOPE,
        DATE,
        "決定の題名",
        DecisionScope::recorded("一段目。二段目。").unwrap(),
    )
    .unwrap();
    decision.add_alias(Alias("D-1".into()));
    decision.set_body("## Context\nthe context\n\n## Decision\nwe chose");
    let mut question =
        Node::question(id(NodeKind::Question, "0001"), SCOPE, DATE, "論点の題名").unwrap();
    question.add_alias(Alias("Q-1".into()));
    if let NodeData::Question(data) = question.data_mut() {
        data.closure = Some(Closure::Fact);
        data.evidence = Some("測定で決まった".into());
        data.decider = Some("master".into());
        data.options = vec!["a".into(), "b".into()];
    }
    let mut criterion =
        Node::criterion(id(NodeKind::Criterion, "0004"), SCOPE, DATE, "ACの題名").unwrap();
    criterion.set_body("測り方の本文");
    if let NodeData::Criterion(data) = criterion.data_mut() {
        data.satisfied = true;
        data.evidence = Some("verified".into());
    }
    let mut requirement = Node::requirement(
        id(NodeKind::Requirement, "0003"),
        SCOPE,
        DATE,
        "要求の題名",
        RequirementState::Approved,
    )
    .unwrap();
    requirement.add_alias(Alias("R-1".into()));
    if let NodeData::Requirement(data) = requirement.data_mut() {
        data.reference = Some(Ref("https://example/1".into()));
        data.approval = Some(Approval {
            design: "d".into(),
            heard_by: "m".into(),
            evidence: "e".into(),
            at: DATE.into(),
        });
    }
    requirement.set_free("next_evidence", "次の根拠");
    requirement.link(
        Link::new(
            requirement.id().clone(),
            Relation::ReliesOn,
            decision.id().clone(),
        )
        .unwrap(),
    );
    requirement.link(
        Link::new(
            requirement.id().clone(),
            Relation::Targets,
            criterion.id().clone(),
        )
        .unwrap(),
    );
    let mut need = Node::need(id(NodeKind::Need, "0005"), SCOPE, DATE, "ニーズの題名").unwrap();
    need.link(
        Link::new(
            need.id().clone(),
            Relation::FiledAs,
            requirement.id().clone(),
        )
        .unwrap(),
    );
    seed(
        &mut repo,
        &[decision, question, criterion, requirement, need],
    );
    repo
}

#[test]
fn publish_has_a_header_a_reading_and_no_master_sections() {
    let repo = ledger();
    let text = render(&repo, None);
    for marker in [
        "# gy の公開物",
        "- seq: ",
        "- scope: ",
        "- since: ",
        "- 書き手: piko",
        "- 正本: /ledger",
        "## 読み方",
        "## 記録",
        "## 履歴",
        "## 診断",
    ] {
        assert!(text.contains(marker), "missing {marker}:\n{text}");
    }
    assert!(!text.contains("いま判断待ち"), "{text}");
    assert!(!text.contains("未決の論点"), "{text}");
}

#[test]
fn publish_writes_every_node_verbatim_with_titled_references() {
    let repo = ledger();
    let text = render(&repo, None);
    // Decision, question, requirement, and criterion records.
    assert!(text.contains("decision_scope: 一段目。二段目。"), "{text}");
    assert!(text.contains("## Decision\nwe chose"), "{text}");
    assert!(text.contains("閉じ方: 事実（測定で決まった）"), "{text}");
    assert!(text.contains("satisfied: true (verified)"), "{text}");
    assert!(
        text.contains("承認: design=d, heard_by=m, evidence=e, at=2026-09-15"),
        "{text}"
    );
    assert!(text.contains("(R-1, https://example/1)"), "{text}");
    assert!(text.contains("next_evidence: 次の根拠"), "{text}");
    // Both directions of an edge, each with its target's title.
    assert!(text.contains("relies-on d-0002 (D-1) 決定の題名"), "{text}");
    assert!(
        text.contains("relied-on-by r-0003 (R-1) 要求の題名"),
        "{text}"
    );
    // No line is an ID alone.
    for id in ["d-0002", "q-0001", "r-0003", "ac-0004", "n-0005"] {
        assert!(!text.lines().any(|line| line.trim() == id), "{text}");
    }
}

#[test]
fn publish_keeps_a_closed_need_reason() {
    let mut repo = repo();
    let mut need = Node::need(id(NodeKind::Need, "0007"), SCOPE, DATE, "閉じたニーズ").unwrap();
    if let NodeData::Need(data) = need.data_mut() {
        data.closed = Some(gy_ledger::Closed {
            by: gy_ledger::ClosedBy::External,
            evidence: "既存で満たした".into(),
        });
    }
    seed(&mut repo, &[need]);
    let text = render(&repo, None);
    assert!(
        text.contains("閉じた理由: 外部（既存で満たした）"),
        "{text}"
    );
}

#[test]
fn publish_filters_by_since() {
    let mut repo = repo();
    seed(
        &mut repo,
        &[Node::need(id(NodeKind::Need, "0001"), SCOPE, DATE, "古いニーズ").unwrap()],
    );
    seed(
        &mut repo,
        &[Node::need(id(NodeKind::Need, "0002"), SCOPE, DATE, "新しいニーズ").unwrap()],
    );

    let all = render(&repo, None);
    assert!(
        all.contains("古いニーズ") && all.contains("新しいニーズ"),
        "{all}"
    );
    let recent = render(&repo, Some(1));
    assert!(!recent.contains("古いニーズ"), "{recent}");
    assert!(recent.contains("新しいニーズ"), "{recent}");
}

#[test]
fn publish_lists_history_and_diagnostics_deterministically() {
    let repo = ledger();
    let text = render(&repo, None);
    assert!(text.contains("| seq |"), "{text}");
    assert!(
        text.lines()
            .any(|line| line.starts_with("| 1 |") && line.contains("決定の題名")),
        "{text}"
    );
    assert!(text.contains("errors: 0"), "{text}");
    assert!(text.contains("warnings: 0"), "{text}");

    let strip = |body: &str| {
        body.lines()
            .filter(|line| !line.starts_with("- 生成:"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(strip(&text), strip(&render(&repo, None)));
}
