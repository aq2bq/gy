use gy_ledger::link::Link;
use gy_ledger::{
    Actor, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Operation, QuestionAdd,
    Relation, Repository, show,
};

const SCOPE: &str = "a";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn add(decider: &str, options: &[&str]) -> QuestionAdd {
    QuestionAdd {
        scope: SCOPE.into(),
        title: "a question".into(),
        decider: decider.into(),
        options: options.iter().map(|option| option.to_string()).collect(),
        body: None,
    }
}

#[test]
fn question_add_creates_a_question() {
    let mut repo = repo();
    let outcome = add("master", &["a", "b"]).run(&mut repo).unwrap();
    let node = repo.get(outcome.id.as_ref().unwrap()).unwrap().unwrap();
    assert_eq!(node.kind(), NodeKind::Question);
    match node.data() {
        NodeData::Question(data) => {
            assert_eq!(data.decider.as_deref(), Some("master"));
            assert_eq!(data.options.len(), 2);
        }
        _ => panic!("not a question"),
    }
    assert_eq!(outcome.changed, ["created", "decider", "options"]);
}

#[test]
fn question_add_without_a_decider_is_rejected() {
    let mut repo = repo();
    assert!(add("  ", &["a", "b"]).run(&mut repo).is_err());
    assert!(repo.all().unwrap().is_empty());
}

#[test]
fn question_add_with_fewer_than_two_options_is_rejected() {
    let mut repo = repo();
    assert!(add("master", &["a"]).run(&mut repo).is_err());
    assert!(repo.all().unwrap().is_empty());
}

#[test]
fn a_question_without_a_waiting_need_reports_it() {
    let mut repo = repo();
    let outcome = add("master", &["a", "b"]).run(&mut repo).unwrap();
    let id = outcome.id.clone().unwrap();
    assert_eq!(
        outcome.missing,
        [
            "本文（選択肢の根拠）",
            "待つニーズ（waits-on）か生んだ要求（raised）"
        ]
    );
    assert_eq!(
        outcome.next,
        [
            format!("edit {id} --body-file … --reason …"),
            format!("question close {id} --by … --evidence …"),
            format!("decide … --closes {id}"),
            format!("link <need> waits-on {id}")
        ]
    );

    // A need that waits on it settles the gap and drops the suggestion.
    let need = Node::need(
        NodeId::from_hash(NodeKind::Need, "0002").unwrap(),
        SCOPE,
        "2026-09-15T00:00:00Z",
        "a need",
    )
    .unwrap();
    repo.transaction("seed", "test", |repo| repo.put(&need))
        .unwrap();
    Link {
        from: need.id().clone(),
        relation: Relation::WaitsOn,
        to: id.clone(),
        mark: None,
        remove: false,
    }
    .run(&mut repo)
    .unwrap();
    let text = show(&repo, &[id.to_string()], false).unwrap()[0].to_string();
    assert!(!text.contains("待つニーズ"), "{text}");
}
