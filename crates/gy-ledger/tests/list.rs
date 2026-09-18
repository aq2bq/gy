use gy_ledger::{
    Actor, Filter, FormatVersion, Link, Listing, LogRow, MemoryStore, Node, NodeData, NodeId,
    NodeKind, Ref, Relation, Repository, RequirementState, Row, Store, list,
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

fn node_rows<S: Store>(repo: &Repository<S>, filter: &Filter) -> Vec<Row> {
    match list(repo, filter).unwrap() {
        Listing::Nodes(rows) => rows,
        Listing::History(_) => panic!("expected node rows"),
    }
}

fn log_rows<S: Store>(repo: &Repository<S>, filter: &Filter) -> Vec<LogRow> {
    match list(repo, filter).unwrap() {
        Listing::History(rows) => rows,
        Listing::Nodes(_) => panic!("expected history rows"),
    }
}

#[test]
fn list_filters_by_kind() {
    let mut repo = repo();
    let need = Node::need(id(NodeKind::Need, "0001"), SCOPE, DATE, "a need").unwrap();
    let criterion =
        Node::criterion(id(NodeKind::Criterion, "0002"), SCOPE, DATE, "a criterion").unwrap();
    seed(&mut repo, &[need, criterion]);

    let filter = Filter {
        kind: Some(NodeKind::Need),
        ..Filter::default()
    };
    let rows = node_rows(&repo, &filter);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, NodeKind::Need);
    assert_eq!(rows[0].status.as_deref(), Some("open"));
    assert_eq!(rows[0].title, "a need");
    assert_eq!(rows[0].scope, SCOPE);
    assert_eq!(rows[0].created, DATE);
}

#[test]
fn list_filters_by_status_and_rejects_an_unknown_one() {
    let mut repo = repo();
    let need = Node::need(id(NodeKind::Need, "0003"), SCOPE, DATE, "a need").unwrap();
    let criterion =
        Node::criterion(id(NodeKind::Criterion, "0004"), SCOPE, DATE, "a criterion").unwrap();
    seed(&mut repo, &[need, criterion]);

    let open = Filter {
        status: Some("open".into()),
        ..Filter::default()
    };
    assert_eq!(node_rows(&repo, &open).len(), 1);
    let unsatisfied = Filter {
        status: Some("unsatisfied".into()),
        ..Filter::default()
    };
    let rows = node_rows(&repo, &unsatisfied);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, NodeKind::Criterion);
    let weird = Filter {
        status: Some("weird".into()),
        ..Filter::default()
    };
    assert!(list(&repo, &weird).is_err());
}

#[test]
fn list_filters_a_done_need_by_its_derived_state() {
    let mut repo = repo();
    let requirement = Node::requirement(
        id(NodeKind::Requirement, "0020"),
        SCOPE,
        DATE,
        "a done requirement",
        RequirementState::Done,
    )
    .unwrap();
    let requirement_id = requirement.id().clone();
    let mut need = Node::need(id(NodeKind::Need, "0021"), SCOPE, DATE, "a done need").unwrap();
    need.link(Link::new(need.id().clone(), Relation::FiledAs, requirement_id).unwrap());
    let need_id = need.id().clone();
    seed(&mut repo, &[requirement, need]);

    let filter = Filter {
        kind: Some(NodeKind::Need),
        status: Some("done".into()),
        ..Filter::default()
    };
    let rows = node_rows(&repo, &filter);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, need_id.to_string());
    assert_eq!(rows[0].status.as_deref(), Some("done"));
}

#[test]
fn list_filters_by_targets_and_grep() {
    let mut repo = repo();
    let criterion = Node::criterion(
        id(NodeKind::Criterion, "0005"),
        SCOPE,
        DATE,
        "the criterion",
    )
    .unwrap();
    let criterion_id = criterion.id().clone();
    let mut requirement = Node::requirement(
        id(NodeKind::Requirement, "0006"),
        SCOPE,
        DATE,
        "alpha request",
        RequirementState::Filed,
    )
    .unwrap();
    if let NodeData::Requirement(data) = requirement.data_mut() {
        data.reference = Some(Ref("https://example/9".into()));
    }
    requirement.link(
        Link::new(
            requirement.id().clone(),
            Relation::Targets,
            criterion_id.clone(),
        )
        .unwrap(),
    );
    let requirement_id = requirement.id().clone();
    let other = Node::need(id(NodeKind::Need, "0007"), SCOPE, DATE, "beta").unwrap();
    seed(&mut repo, &[criterion, requirement, other]);

    let by_target = Filter {
        targets: Some(criterion_id),
        ..Filter::default()
    };
    let rows = node_rows(&repo, &by_target);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, requirement_id.to_string());
    assert_eq!(rows[0].reference.as_deref(), Some("https://example/9"));

    let by_title = Filter {
        grep: Some("alpha".into()),
        ..Filter::default()
    };
    assert_eq!(node_rows(&repo, &by_title).len(), 1);
    let none = Filter {
        grep: Some("gamma".into()),
        ..Filter::default()
    };
    assert!(node_rows(&repo, &none).is_empty());
}

#[test]
fn list_returns_history_for_actor_and_since() {
    let mut repo = repo();
    seed(
        &mut repo,
        &[Node::need(id(NodeKind::Need, "0008"), SCOPE, DATE, "one").unwrap()],
    );
    seed(
        &mut repo,
        &[Node::need(id(NodeKind::Need, "0009"), SCOPE, DATE, "two").unwrap()],
    );

    let actor = Filter {
        actor: Some("piko".into()),
        ..Filter::default()
    };
    let rows = log_rows(&repo, &actor);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].seq, 1);
    assert_eq!(rows[0].actor, "piko");
    assert_eq!(rows[0].why, "seed");
    assert_eq!(rows[0].source, "test");

    let since = Filter {
        since: Some(1),
        ..Filter::default()
    };
    let rows = log_rows(&repo, &since);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].seq, 2);
}

#[test]
fn history_entries_of_one_transaction_share_the_seq() {
    let mut repo = repo();
    seed(
        &mut repo,
        &[
            Node::need(id(NodeKind::Need, "0010"), SCOPE, DATE, "one").unwrap(),
            Node::need(id(NodeKind::Need, "0011"), SCOPE, DATE, "two").unwrap(),
        ],
    );
    let since = Filter {
        since: Some(0),
        ..Filter::default()
    };
    let rows = log_rows(&repo, &since);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].seq, 1);
    assert_eq!(rows[1].seq, 1);
}

#[test]
fn list_is_empty_when_nothing_matches() {
    let repo = repo();
    assert!(node_rows(&repo, &Filter::default()).is_empty());
    assert_eq!(list(&repo, &Filter::default()).unwrap().to_string(), "");
}

#[test]
fn list_marks_questions_nobody_waits_on() {
    let mut repo = repo();
    let question =
        Node::question(id(NodeKind::Question, "0030"), SCOPE, DATE, "a question").unwrap();
    let need = Node::need(id(NodeKind::Need, "0031"), SCOPE, DATE, "a need").unwrap();
    seed(&mut repo, &[question, need]);

    let rows = node_rows(&repo, &Filter::default());
    let asked = rows.iter().find(|row| row.id == "q-0030").unwrap();
    assert!(asked.unwaited);
    assert_eq!(asked.created, DATE);
    let wanted = rows.iter().find(|row| row.id == "n-0031").unwrap();
    assert!(!wanted.unwaited);
}
