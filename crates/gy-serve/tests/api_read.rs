use gy_ledger::link::Link;
use gy_ledger::{
    Actor, Alias, DecisionScope, Edit, Filter, FormatVersion, Listing, MemoryStore, Node, NodeData,
    NodeId, NodeKind, Operation, Relation, Repository, RequirementState, list, show,
};
use gy_serve::api::route;
use gy_serve::http::Request;

const DATE: &str = "2026-09-15";

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

/// Two needs' worth of kinds in two scopes: a need targeting a criterion, a
/// question that names a decider, a decision with an alias, a filed
/// requirement, and twelve criteria that all match one word.
fn ledger() -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string(), "b".to_string()]);
    let mut nodes = vec![
        Node::need(id(NodeKind::Need, "0001"), "a", DATE, "a need").unwrap(),
        Node::criterion(id(NodeKind::Criterion, "0002"), "a", DATE, "measures well").unwrap(),
        Node::criterion(id(NodeKind::Criterion, "0003"), "b", DATE, "measures later").unwrap(),
        Node::question(id(NodeKind::Question, "0004"), "a", DATE, "which way").unwrap(),
        Node::decision(
            id(NodeKind::Decision, "0005"),
            "a",
            DATE,
            "publish the page",
            DecisionScope::recorded("scope").unwrap(),
        )
        .unwrap(),
        Node::requirement(
            id(NodeKind::Requirement, "0006"),
            "a",
            DATE,
            "a requirement",
            RequirementState::Filed,
        )
        .unwrap(),
    ];
    if let NodeData::Question(data) = nodes[3].data_mut() {
        data.decider = Some("master".to_string());
        data.options = vec!["left".to_string(), "right".to_string()];
    }
    nodes[4].add_alias(Alias("D-85".to_string()));
    for index in 0..12 {
        let hash = format!("1{index:03}");
        let node =
            Node::criterion(id(NodeKind::Criterion, &hash), "a", DATE, "duplicated").unwrap();
        nodes.push(node);
    }
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    Link {
        from: id(NodeKind::Need, "0001"),
        relation: Relation::FiledAs,
        to: id(NodeKind::Requirement, "0006"),
        mark: None,
        remove: false,
    }
    .run(&mut repo)
    .unwrap();
    Edit {
        id: id(NodeKind::Decision, "0005"),
        reason: "tidy".to_string(),
        title: Some("publish the list page".to_string()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();
    repo
}

fn answer(repo: &Repository<MemoryStore>, path: &str, query: Option<&str>) -> serde_json::Value {
    let res = route(repo, &Request::new("GET", path, query, &[]));
    serde_json::from_slice(&res.body).unwrap()
}

fn status(path: &str, query: Option<&str>) -> u16 {
    route(&ledger(), &Request::new("GET", path, query, &[])).status
}

fn ids(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect()
}

/// The view's rows for one filter, as ids.
fn view_ids(repo: &Repository<MemoryStore>, filter: &Filter) -> Vec<String> {
    match list(repo, filter).unwrap() {
        Listing::Nodes(rows) => rows.iter().map(|row| row.id.clone()).collect(),
        Listing::History(_) => Vec::new(),
    }
}

#[test]
fn list_matches_the_view() {
    let repo = ledger();
    let filter = Filter {
        kind: Some(NodeKind::Need),
        ..Default::default()
    };
    assert_eq!(
        ids(&answer(&repo, "/api/list", Some("kind=Need"))["rows"]),
        view_ids(&repo, &filter)
    );

    let all = answer(&repo, "/api/list", None);
    let scoped = answer(&repo, "/api/list", Some("scope=b"));
    assert!(!all["rows"].as_array().unwrap().is_empty());
    assert!(scoped["rows"].as_array().unwrap().len() < all["rows"].as_array().unwrap().len());

    let grep = Filter {
        grep: Some("duplicated".to_string()),
        ..Default::default()
    };
    let rows = answer(&repo, "/api/list", Some("q=duplicated"));
    assert_eq!(ids(&rows["rows"]), view_ids(&repo, &grep));
    assert_eq!(rows["rows"].as_array().unwrap().len(), 12);

    assert_eq!(status("/api/list", Some("kind=Foo")), 400);
    assert_eq!(status("/api/list", Some("status=nonsense")), 400);
}

#[test]
fn node_names_its_edges_and_history() {
    let repo = ledger();
    let view = show(&repo, &["n-0001".to_string()], true)
        .unwrap()
        .remove(0);
    let json = answer(&repo, "/api/node/n-0001", None);
    assert_eq!(json["id"], view.id);
    assert_eq!(json["title"], view.title);
    assert_eq!(json["edges"].as_array().unwrap().len(), view.edges.len());

    let edges = json["edges"].as_array().unwrap();
    let peer = edges.iter().find(|edge| edge["to"] == "r-0006").unwrap();
    assert_eq!(peer["title"], "a requirement");
    assert_eq!(peer["kind"], "Requirement");

    let history = json["history"].as_array().unwrap();
    assert_eq!(history.len(), 2);
    assert!(history.iter().all(|row| row["node"] == "n-0001"));
    assert!(history[0]["seq"].as_u64().unwrap() > history[1]["seq"].as_u64().unwrap());

    let aliased = answer(&repo, "/api/node/D-85", None);
    assert_eq!(aliased["id"], "d-0005");
    assert_eq!(aliased["aliases"][0], "D-85");
    assert_eq!(status("/api/node/nope", None), 404);
}

#[test]
fn search_matches_id_alias_and_title() {
    let repo = ledger();
    let exact = answer(&repo, "/api/search", Some("q=D-85"));
    assert_eq!(exact["hits"][0]["id"], "d-0005");
    assert_eq!(exact["hits"].as_array().unwrap().len(), 1);

    let title = answer(&repo, "/api/search", Some("q=publish"));
    assert!(
        ids(&title["hits"]).contains(&"d-0005".to_string()),
        "{title}"
    );

    let empty = answer(&repo, "/api/search", Some("q="));
    assert!(empty["hits"].as_array().unwrap().is_empty());

    let capped = answer(&repo, "/api/search", Some("q=duplicated"));
    assert_eq!(capped["hits"].as_array().unwrap().len(), 10);
}
