use gy_ledger::{Actor, CriterionAdd, FormatVersion, MemoryStore, NodeKind, Operation, Repository};

const SCOPE: &str = "a";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn add(title: &str) -> CriterionAdd {
    CriterionAdd {
        scope: SCOPE.into(),
        title: title.into(),
    }
}

#[test]
fn criterion_add_creates_a_criterion() {
    let mut repo = repo();
    let outcome = add("Retries must not duplicate").run(&mut repo).unwrap();
    let node = repo.get(outcome.id.as_ref().unwrap()).unwrap().unwrap();
    assert_eq!(node.kind(), NodeKind::Criterion);
    assert_eq!(outcome.changed, ["created"]);
}

#[test]
fn criterion_add_with_a_blank_title_is_rejected() {
    let mut repo = repo();
    assert!(add("   ").run(&mut repo).is_err());
    assert!(repo.all().unwrap().is_empty());
}
