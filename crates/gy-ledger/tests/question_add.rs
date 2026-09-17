use gy_ledger::{
    Actor, FormatVersion, MemoryStore, NodeData, NodeKind, Operation, QuestionAdd, Repository,
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
