use gy_ledger::{
    Actor, FileStore, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Ref,
    Repository, RequirementState,
};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15";

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        SCOPE,
        DATE,
        "a criterion",
    )
    .unwrap()
}

#[test]
fn put_then_get_and_all() {
    let mut repo = repo();
    let (one, two) = (criterion("0001"), criterion("0002"));
    repo.transaction("add", "test", |repo| {
        repo.put(&one)?;
        repo.put(&two)
    })
    .unwrap();
    assert_eq!(repo.get(one.id()).unwrap().unwrap(), one);
    assert_eq!(repo.all().unwrap().len(), 2);
}

#[test]
fn remove_drops_the_node() {
    let mut repo = repo();
    let node = criterion("0001");
    repo.transaction("add", "test", |repo| repo.put(&node))
        .unwrap();
    repo.transaction("drop", "test", |repo| repo.remove(node.id()))
        .unwrap();
    assert!(repo.get(node.id()).unwrap().is_none());
}

#[test]
fn resolve_handles_full_ids_zero_padding_and_aliases() {
    let mut repo = repo();
    let mut node = criterion("0001");
    node.add_alias(gy_ledger::Alias("AC-7".into()));
    repo.transaction("add", "test", |repo| repo.put(&node))
        .unwrap();
    assert_eq!(repo.resolve("ac-0001").unwrap(), node.id().clone());
    assert_eq!(repo.resolve("AC-1").unwrap(), node.id().clone());
    assert_eq!(repo.resolve("ac-7").unwrap(), node.id().clone());
    assert!(repo.resolve("ac-9999").is_err());
}

#[test]
fn resolve_reports_candidates_when_ambiguous() {
    let mut repo = repo();
    let (mut first, mut second) = (criterion("0001"), criterion("0002"));
    first.add_alias(gy_ledger::Alias("AC-7".into()));
    second.add_alias(gy_ledger::Alias("AC-07".into()));
    repo.transaction("add", "test", |repo| {
        repo.put(&first)?;
        repo.put(&second)
    })
    .unwrap();
    let error = repo.resolve("ac-7").unwrap_err().to_string();
    assert!(
        error.contains("ac-0001") && error.contains("ac-0002"),
        "{error}"
    );
}

#[test]
fn a_failed_transaction_writes_nothing() {
    let mut repo = repo();
    let node = criterion("0001");
    let result: gy_ledger::Result<()> = repo.transaction("add", "test", |repo| {
        repo.put(&node)?;
        Err(gy_ledger::Error::invalid("stop"))
    });
    assert!(result.is_err());
    assert!(repo.get(node.id()).unwrap().is_none());
}

#[test]
fn a_file_store_survives_reopen() {
    let temp = tempfile::tempdir().unwrap();
    gy_ledger::format::write(temp.path(), FormatVersion::CURRENT).unwrap();
    let actor = |_: &str| Some("piko".to_string());
    let mut repo = Repository::new(FileStore::open_with(temp.path(), actor).unwrap());
    let node = criterion("0001");
    repo.transaction("add", "test", |repo| repo.put(&node))
        .unwrap();
    drop(repo);
    let reopened = Repository::new(FileStore::open_with(temp.path(), actor).unwrap());
    assert_eq!(reopened.get(node.id()).unwrap().unwrap(), node);
}

fn requirement(hash: &str, reference: &str) -> Node {
    let mut node = Node::requirement(
        NodeId::from_hash(NodeKind::Requirement, hash).unwrap(),
        SCOPE,
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    if let NodeData::Requirement(data) = node.data_mut() {
        data.reference = Some(Ref(reference.into()));
    }
    node
}

#[test]
fn resolve_finds_a_requirement_by_its_reference() {
    let mut repo = repo();
    let node = requirement("0001", "https://tracker/6027");
    repo.transaction("add", "test", |repo| repo.put(&node))
        .unwrap();
    assert_eq!(
        repo.resolve("https://tracker/6027").unwrap(),
        node.id().clone()
    );
    assert_eq!(repo.resolve("6027").unwrap(), node.id().clone());
    assert!(repo.resolve("9999").is_err());
}

#[test]
fn resolve_reports_ambiguous_references() {
    let mut repo = repo();
    let (first, second) = (
        requirement("0001", "https://one/6027"),
        requirement("0002", "https://two/6027"),
    );
    repo.transaction("add", "test", |repo| {
        repo.put(&first)?;
        repo.put(&second)
    })
    .unwrap();
    let error = repo.resolve("6027").unwrap_err().to_string();
    assert!(
        error.contains("r-0001") && error.contains("r-0002"),
        "{error}"
    );
}
