//! The three reading routes' shapes and meanings (r-44c7, d-6d90): overview,
//! cross, and the node path. The judgements come from gy-ledger, so these tests
//! pin what the screens are promised, not how it is derived.
use gy_ledger::{
    Actor, Alias, Completion, DecisionScope, FormatVersion, Link, MemoryStore, Node, NodeData,
    NodeId, NodeKind, Relation, Repository, RequirementState,
};
use gy_serve::api::route;
use gy_serve::http::Request;
use serde_json::Value;

const DATE: &str = "2026-09-15T00:00:00Z";

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn link(from: &mut Node, relation: Relation, to: NodeId) {
    from.link(Link::new(from.id().clone(), relation, to).unwrap());
}

fn marked(from: &mut Node, relation: Relation, to: NodeId, mark: &str) {
    let held = Link::new(from.id().clone(), relation, to)
        .unwrap()
        .with_mark(Some(mark.to_string()));
    from.link(held);
}

/// Two needs (one blocked, one waiting to close), two questions (one nobody
/// asks), three criteria (one met, one overdue, one nobody aims at), a decision
/// with a successor, and a requirement that holds a question and a criterion.
fn ledger() -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".into(), "b".into()]);
    let note = || DecisionScope::recorded("s").unwrap();
    let mut first = Node::need(id(NodeKind::Need, "0001"), "a", DATE, "the first need").unwrap();
    let mut second = Node::need(id(NodeKind::Need, "0002"), "b", DATE, "the second need").unwrap();
    let mut asked = Node::question(id(NodeKind::Question, "0003"), "a", DATE, "which way").unwrap();
    let mut unasked = Node::question(
        id(NodeKind::Question, "0009"),
        "a",
        DATE,
        "nobody asks this",
    )
    .unwrap();
    if let NodeData::Question(data) = asked.data_mut() {
        data.decider = Some("master".into());
        data.options = vec!["left".into(), "right".into()];
    }
    unasked.add_alias(Alias("Q-9".into()));
    let mut decision = Node::decision(
        id(NodeKind::Decision, "0004"),
        "a",
        DATE,
        "the first",
        note(),
    )
    .unwrap();
    let mut later = Node::decision(
        id(NodeKind::Decision, "0008"),
        "a",
        DATE,
        "the later",
        note(),
    )
    .unwrap();
    let mut dated =
        Node::criterion(id(NodeKind::Criterion, "0005"), "a", DATE, "a dated check").unwrap();
    dated.set_free("due", "2020-01-01");
    dated.set_free("responsible", "master");
    dated.set_free("then", "ask again");
    let met = Node::criterion(id(NodeKind::Criterion, "0006"), "b", DATE, "a met check").unwrap();
    // A done requirement covers the criterion, so the model lets it be read as
    // satisfied (rule I1): a criterion met without coverage is refused.
    let mut cover = Node::requirement(
        id(NodeKind::Requirement, "0011"),
        "b",
        DATE,
        "the coverage",
        RequirementState::Done,
    )
    .unwrap();
    if let NodeData::Requirement(data) = cover.data_mut() {
        data.completion = Some(Completion {
            evidence: "it works".into(),
            at: DATE.into(),
        });
    }
    link(
        &mut cover,
        Relation::Targets,
        id(NodeKind::Criterion, "0006"),
    );
    let homeless = Node::criterion(
        id(NodeKind::Criterion, "0010"),
        "a",
        DATE,
        "a homeless check",
    )
    .unwrap();
    let mut requirement = Node::requirement(
        id(NodeKind::Requirement, "0007"),
        "a",
        DATE,
        "a requirement",
        RequirementState::Filed,
    )
    .unwrap();
    link(
        &mut first,
        Relation::FiledAs,
        id(NodeKind::Requirement, "0007"),
    );
    link(
        &mut first,
        Relation::Targets,
        id(NodeKind::Criterion, "0005"),
    );
    link(
        &mut first,
        Relation::WaitsOn,
        id(NodeKind::Question, "0003"),
    );
    link(&mut first, Relation::DependsOn, id(NodeKind::Need, "0002"));
    link(
        &mut second,
        Relation::Targets,
        id(NodeKind::Criterion, "0006"),
    );
    link(&mut decision, Relation::Serves, id(NodeKind::Need, "0001"));
    marked(
        &mut later,
        Relation::Supersedes,
        id(NodeKind::Decision, "0004"),
        "the passage",
    );
    link(
        &mut requirement,
        Relation::Raised,
        id(NodeKind::Question, "0003"),
    );
    link(
        &mut requirement,
        Relation::ReliesOn,
        id(NodeKind::Decision, "0004"),
    );
    link(
        &mut requirement,
        Relation::Targets,
        id(NodeKind::Criterion, "0005"),
    );
    let nodes = [
        first,
        second,
        asked,
        decision,
        dated,
        met.clone(),
        unasked,
        requirement,
        later,
        homeless,
        cover.clone(),
    ];
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    let mut met = met;
    if let NodeData::Criterion(data) = met.data_mut() {
        data.satisfied = true;
        data.satisfied_at = Some(DATE.into());
    }
    repo.transaction("the check passed", "test", |repo| repo.put(&met))
        .unwrap();
    repo
}

fn get(path: &str, query: Option<&str>) -> Value {
    let response = route(&ledger(), &Request::new("GET", path, query, &[]), "gy");
    assert_eq!(response.status, 200, "{path}");
    serde_json::from_slice(&response.body).unwrap()
}

fn ids(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect()
}

fn overview() -> Value {
    get("/api/overview", None)
}

#[test]
fn the_overview_counts_what_the_graph_says() {
    let json = overview();
    assert!(json["seq"].as_u64().unwrap() >= 1);
    assert!(json["at"].as_str().unwrap().ends_with('Z'));
    assert_eq!(
        json["counts"],
        serde_json::json!({
            "open_needs": 2,
            "blocked_needs": 1,
            "open_questions": 2,
            "unmet_criteria": 2,
            "closed_needs": 0,
        })
    );
}

#[test]
fn the_overview_puts_every_open_need_where_it_stands() {
    let json = overview();
    assert_eq!(ids(&json, "needs"), ["n-0001", "n-0002"]);
    let first = &json["needs"][0];
    assert_eq!(first["stage"], "build");
    assert_eq!(first["blocked"], true);
    assert_eq!(first["scope"], "a");
    assert!(first["last_moved"].as_str().unwrap().ends_with('Z'));
    assert_eq!(
        first["requirements"],
        serde_json::json!({"filed": 1, "approved": 0, "done": 0, "cancelled": 0})
    );
    assert_eq!(
        first["criteria"],
        serde_json::json!({"satisfied": 0, "unsatisfied": 1, "withdrawn": 0})
    );
    // The open question comes with its decider; the need has none.
    let waiting = first["waiting"].as_array().unwrap();
    assert_eq!(waiting.len(), 2);
    let question = waiting
        .iter()
        .find(|row| row["id"] == "q-0003")
        .expect("the question it waits on");
    assert_eq!(question["kind"], "Question");
    assert_eq!(question["decider"], "master");
    let need = waiting
        .iter()
        .find(|row| row["id"] == "n-0002")
        .expect("the need it depends on");
    assert!(need.get("decider").is_none());

    // The second need has its criteria met and no requirement: it waits to close.
    assert_eq!(json["needs"][1]["stage"], "close");
    assert_eq!(json["needs"][1]["blocked"], false);
}

#[test]
fn the_overview_names_the_roots_of_the_waits() {
    let json = overview();
    let blockers = json["blockers"].as_array().unwrap();
    assert_eq!(blockers.len(), 2);
    // One need each, ties by id, so the need root comes before the question.
    assert_eq!(blockers[1]["id"], "q-0003");
    assert_eq!(blockers[1]["kind"], "Question");
    assert_eq!(blockers[1]["decider"], "master");
    assert_eq!(blockers[1]["needs"], serde_json::json!(["n-0001"]));
    assert_eq!(blockers[1]["via"], serde_json::json!([]));
    assert_eq!(blockers[0]["id"], "n-0002");
    assert_eq!(blockers[0]["kind"], "Need");
}

#[test]
fn the_overview_names_the_deciders_the_overdue_and_the_unheld() {
    let json = overview();
    // Two open questions: one names master, one names nobody.
    assert_eq!(
        json["deciders"],
        serde_json::json!([{"name": "", "count": 1}, {"name": "master", "count": 1}])
    );
    let overdue = json["overdue"].as_array().unwrap();
    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue[0]["id"], "ac-0005");
    assert_eq!(overdue[0]["due"], "2020-01-01");
    assert_eq!(overdue[0]["responsible"], "master");
    assert_eq!(overdue[0]["then"], "ask again");
    // The question nobody waits on, then the criterion nobody aims at.
    assert_eq!(ids(&json, "unheld"), ["q-0009", "ac-0010"]);
    assert_eq!(json["unheld"][0]["state"], "open");
    assert_eq!(json["unheld"][1]["state"], "unsatisfied");
}

#[test]
fn the_overview_has_every_week_the_record_spans() {
    let json = overview();
    let weeks = json["weeks"].as_array().unwrap();
    assert!(!weeks.is_empty());
    assert!(weeks[0]["from"].as_str().unwrap() <= "2026-09-15");
    let opened: u64 = weeks
        .iter()
        .map(|week| week["opened"].as_u64().unwrap())
        .sum();
    assert_eq!(opened, 2);
}

#[test]
fn the_overview_gives_each_scope_its_needs_and_its_pulse() {
    let json = overview();
    let scopes = json["scopes"].as_array().unwrap();
    let names: Vec<&str> = scopes
        .iter()
        .map(|scope| scope["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["a", "b"]);
    assert_eq!(scopes[0]["open"], 1);
    assert_eq!(scopes[0]["blocked"], 1);
    let writes: u64 = scopes[0]["pulse"]
        .as_array()
        .unwrap()
        .iter()
        .map(|day| day["writes"].as_u64().unwrap())
        .sum();
    // One write touched scope a; it is counted once, not once per node.
    assert_eq!(writes, 1);
}

#[test]
fn a_cross_cut_carries_its_nodes_edges_and_scopes() {
    let json = get("/api/cross", Some("scope=a"));
    assert_eq!(json["capped"], false);
    let nodes = json["nodes"].as_array().unwrap();
    let found: Vec<&str> = nodes
        .iter()
        .map(|node| node["id"].as_str().unwrap())
        .collect();
    // Scope a, by kind: the needs, the questions, the decisions, the
    // requirement, the criteria.
    assert_eq!(
        found,
        [
            "n-0001", "q-0003", "q-0009", "d-0004", "d-0008", "r-0007", "ac-0005", "ac-0010"
        ]
    );
    let first = &nodes[0];
    assert_eq!(first["kind"], "Need");
    assert_eq!(first["state"], "open");
    assert_eq!(first["created"], DATE);
    assert_eq!(first["title"], "the first need");
    // Edges only where both ends are in the cut, in the stored direction.
    let edges = json["edges"].as_array().unwrap();
    assert!(edges.contains(&serde_json::json!({
        "from": "n-0001", "to": "r-0007", "relation": "filed-as"
    })));
    assert!(edges.contains(&serde_json::json!({
        "from": "r-0007", "to": "d-0004", "relation": "relies-on"
    })));
    // An end outside the cut takes its edge with it: the need it depends on
    // lives in scope b.
    assert!(!edges.iter().any(|edge| edge["to"] == "n-0002"));
    assert!(first["degree"].as_u64().unwrap() >= 3);
    let scopes = json["scopes"].as_array().unwrap();
    assert_eq!(scopes.len(), 1);
    let kinds = scopes[0]["kinds"].as_array().unwrap();
    let needs = kinds
        .iter()
        .find(|kind| kind["kind"] == "Need")
        .expect("a kind's count");
    assert_eq!(
        (needs["total"].as_u64(), needs["open"].as_u64()),
        (Some(1), Some(1))
    );
    assert!(json.get("weeks").is_none());
    assert!(json.get("events").is_none());
}

#[test]
fn a_cross_cut_narrowed_by_a_word_and_a_day_lists_what_happened() {
    let json = get("/api/cross", Some("q=check&from=2026-09-15&to=2026-09-15"));
    assert_eq!(json["capped"], false);
    assert_eq!(ids(&json, "nodes"), ["ac-0005", "ac-0010", "ac-0006"]);
    let events = json["events"].as_array().unwrap();
    let what = |name: &str| {
        events
            .iter()
            .find(|event| event["what"] == name)
            .map(|event| event["count"].as_u64().unwrap())
            .unwrap_or(0)
    };
    // The cut holds only criteria: one was satisfied that day, and no decision
    // or question is in it to count.
    assert_eq!(what("criterion-satisfied"), 1);
    assert_eq!(what("decided"), 0);
    assert_eq!(what("question-raised"), 0);
}

#[test]
fn an_unfiltered_cross_draws_no_picture_and_gives_the_grid() {
    let json = get("/api/cross", None);
    assert_eq!(json["capped"], true);
    assert!(json.get("nodes").is_none());
    assert!(json.get("edges").is_none());
    let weeks = json["weeks"].as_array().unwrap();
    assert!(weeks.iter().any(|week| week["scope"] == "a"));
    assert!(weeks.iter().all(|week| week["moves"].as_u64().is_some()));
    assert_eq!(json["scopes"].as_array().unwrap().len(), 2);
}

#[test]
fn a_cross_cut_past_nine_hundred_is_capped() {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".into()]);
    let nodes: Vec<Node> = (0..901)
        .map(|index| {
            Node::need(
                id(NodeKind::Need, &format!("{index:04}")),
                "a",
                DATE,
                "a need",
            )
            .unwrap()
        })
        .collect();
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    let response = route(
        &repo,
        &Request::new("GET", "/api/cross", Some("scope=a"), &[]),
        "gy",
    );
    let json: Value = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(json["capped"], true);
    assert!(json.get("nodes").is_none());
    assert!(!json["weeks"].as_array().unwrap().is_empty());
}

#[test]
fn a_cross_refuses_a_day_it_cannot_read() {
    for query in [
        "from=yesterday",
        "to=2026-02-30",
        "from=2026-09-16&to=2026-09-15",
    ] {
        let response = route(
            &ledger(),
            &Request::new("GET", "/api/cross", Some(query), &[]),
            "gy",
        );
        assert_eq!(response.status, 400, "{query}");
    }
}

#[test]
fn the_path_walks_left_and_right_by_relation() {
    let json = get("/api/node/n-0001/path", None);
    assert_eq!(json["depth"], 2);
    assert_eq!(json["center"]["id"], "n-0001");
    assert_eq!(json["center"]["state"], "open");
    // The prerequisites, in relation order: depends-on, then waits-on.
    let up = json["up"].as_array().unwrap();
    assert_eq!(ids(&up[0], "nodes"), ["n-0002", "q-0003"]);
    assert_eq!(up[0]["nodes"][1]["relation"], "waits-on");
    assert_eq!(up[0]["nodes"][1]["parent"], "n-0001");
    assert_eq!(up[0]["hidden"], 0);
    // A second step: the requirement that raised the question, two hops away.
    assert_eq!(ids(&up[1], "nodes"), ["r-0007"]);
    assert_eq!(up[1]["nodes"][0]["parent"], "q-0003");
    // What it works on: the requirement it filed, the criterion it targets, and
    // the decision that serves it — the superseded decision last, as a closed
    // box sorts after the open ones.
    let down = json["down"].as_array().unwrap();
    assert_eq!(ids(&down[0], "nodes"), ["r-0007", "ac-0005", "d-0004"]);
    assert_eq!(down[0]["nodes"][2]["state"], "superseded");
}

#[test]
fn the_path_of_a_requirement_keeps_the_cause_left_and_the_effect_right() {
    let json = get("/api/node/r-0007/path", Some("depth=3"));
    assert_eq!(json["depth"], 3);
    let up = json["up"].as_array().unwrap();
    // The need that filed it and the decision it relies on.
    assert_eq!(ids(&up[0], "nodes"), ["n-0001", "d-0004"]);
    // The question it raised, then the criterion it targets.
    let down = json["down"].as_array().unwrap();
    assert_eq!(ids(&down[0], "nodes"), ["q-0003", "ac-0005"]);
}

#[test]
fn the_path_reads_a_decision_that_was_superseded() {
    let json = get("/api/node/d-0004/path", None);
    assert_eq!(json["center"]["state"], "superseded");
    // The later decision that supersedes it hangs to its right, with the mark.
    let down = json["down"].as_array().unwrap();
    let later = down[0]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"] == "d-0008")
        .expect("the decision that supersedes it");
    assert_eq!(later["mark"], "the passage");
    assert_eq!(later["state"], "live");
    // Its purpose hangs to its left.
    assert_eq!(ids(&json["up"][0], "nodes"), ["n-0001"]);
}

#[test]
fn the_path_reads_an_alias_and_refuses_what_it_cannot_answer() {
    let json = get("/api/node/Q-9/path", None);
    assert_eq!(json["center"]["id"], "q-0009");
    for (path, query, status) in [
        ("/api/node/n-9999/path", None, 404),
        ("/api/node/n-0001/path", Some("depth=4"), 400),
        ("/api/node/n-0001/path", Some("depth=x"), 400),
    ] {
        let response = route(&ledger(), &Request::new("GET", path, query, &[]), "gy");
        assert_eq!(response.status, status, "{path}?{query:?}");
    }
}

#[test]
fn a_step_keeps_ten_and_counts_the_rest() {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".into()]);
    let mut need = Node::need(id(NodeKind::Need, "0001"), "a", DATE, "a need").unwrap();
    for index in 0..12 {
        need.link(
            Link::new(
                need.id().clone(),
                Relation::WaitsOn,
                id(NodeKind::Question, &format!("1{index:03}")),
            )
            .unwrap(),
        );
    }
    let asks: Vec<Node> = (0..12)
        .map(|index| {
            Node::question(
                id(NodeKind::Question, &format!("1{index:03}")),
                "a",
                DATE,
                "a question",
            )
            .unwrap()
        })
        .collect();
    repo.transaction("seed", "test", |repo| {
        for node in std::iter::once(&need).chain(asks.iter()) {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    let response = route(
        &repo,
        &Request::new("GET", "/api/node/n-0001/path", None, &[]),
        "gy",
    );
    let json: Value = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(json["up"][0]["nodes"].as_array().unwrap().len(), 10);
    assert_eq!(json["up"][0]["hidden"], 2);
}
