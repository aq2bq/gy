use gy_ledger::{
    Actor, ClosedBy, CriterionSatisfy, DecisionScope, FormatVersion, MemoryStore, NeedClose, Node,
    NodeId, NodeKind, Operation, QuestionAdd, Repository, ReqAdd,
};
use gy_serve::api::route;
use gy_serve::http::Request;
use serde_json::json;

const DATE: &str = "2026-09-15";

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

/// Two needs (one closed), one question, one decision, two criteria (one
/// satisfied), in two scopes.
fn ledger() -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".into(), "b".into()]);
    let note = DecisionScope::recorded("s").unwrap();
    let nodes = [
        Node::need(id(NodeKind::Need, "0001"), "a", DATE, "a need").unwrap(),
        Node::need(id(NodeKind::Need, "0002"), "b", DATE, "a need").unwrap(),
        Node::question(id(NodeKind::Question, "0003"), "a", DATE, "a question").unwrap(),
        Node::decision(
            id(NodeKind::Decision, "0004"),
            "a",
            DATE,
            "a decision",
            note,
        )
        .unwrap(),
        Node::criterion(id(NodeKind::Criterion, "0005"), "b", DATE, "met").unwrap(),
        Node::criterion(id(NodeKind::Criterion, "0006"), "a", DATE, "unmet").unwrap(),
    ];
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
    let (closed, met) = (id(NodeKind::Need, "0002"), id(NodeKind::Criterion, "0005"));
    let close = NeedClose {
        id: closed,
        by: ClosedBy::Fact,
        evidence: "x".into(),
    };
    let satisfy = CriterionSatisfy {
        id: met,
        evidence: "x".into(),
        revoke: false,
    };
    close.run(&mut repo).unwrap();
    satisfy.run(&mut repo).unwrap();
    repo
}

fn get(path: &str, query: Option<&str>) -> Request {
    Request::new("GET", path, query, &[])
}

fn shell(query: Option<&str>) -> serde_json::Value {
    let res = route(&ledger(), &get("/api/shell", query), "gy");
    assert_eq!(res.status, 200);
    serde_json::from_slice(&res.body).unwrap()
}

#[test]
fn shell_counts_every_kind_and_scope() {
    let json = shell(None);
    assert_eq!(json["scope"], "all");
    assert!(json["seq"].as_u64().unwrap() >= 1);
    assert!(json["at"].as_str().unwrap().starts_with("20"));
    assert_eq!(
        json["kinds"],
        json!([
            {"kind": "Need", "open": 1, "total": 2},
            {"kind": "Question", "open": 1, "total": 1},
            {"kind": "Decision", "open": 0, "total": 1},
            {"kind": "Requirement", "open": 0, "total": 0},
            {"kind": "Criterion", "open": 1, "total": 2},
        ])
    );
    assert_eq!(
        json["scopes"],
        json!([{"name": "a", "count": 4}, {"name": "b", "count": 2}])
    );
}

#[test]
fn shell_filters_by_scope() {
    let json = shell(Some("scope=b"));
    assert_eq!(json["scope"], "b");
    let kinds = json["kinds"].as_array().unwrap();
    let sum = |key: &str| {
        kinds
            .iter()
            .map(|row| row[key].as_u64().unwrap())
            .sum::<u64>()
    };
    assert_eq!((sum("open"), sum("total")), (0, 2));
    assert_eq!(json["scopes"].as_array().unwrap().len(), 2);
}

#[test]
fn other_methods_are_not_allowed() {
    let res = route(
        &ledger(),
        &Request::new("POST", "/api/shell", None, &[]),
        "gy",
    );
    assert_eq!(res.status, 405);
}

#[test]
fn a_traversal_or_unknown_path_is_not_found() {
    for path in ["/assets/../Cargo.toml", "/assets/missing.js", "/nope"] {
        assert_eq!(
            route(&ledger(), &get(path, None), "gy").status,
            404,
            "{path}"
        );
    }
}

#[test]
fn now_matches_the_view() {
    let mut repo = ledger();
    QuestionAdd {
        scope: "a".to_string(),
        title: "who decides".to_string(),
        decider: "master".to_string(),
        options: vec!["x".to_string(), "y".to_string()],
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    ReqAdd {
        scope: "a".to_string(),
        title: "a requirement".to_string(),
        needs: vec![id(NodeKind::Need, "0001")],
        relies_on: Vec::new(),
        targets: Vec::new(),
        reference: None,
        body: None,
    }
    .run(&mut repo)
    .unwrap();

    let view = gy_ledger::now(&repo, None).unwrap();
    let answer = json(&route(&repo, &get("/api/now", None), "gy"));
    assert_eq!(answer["seq"], view.seq);
    assert!(answer["at"].as_str().unwrap().starts_with("20"));
    assert_eq!(
        answer["waiting"].as_array().unwrap().len(),
        view.waiting.len()
    );
    assert_eq!(json_ids(&answer["in_progress"]), row_ids(&view.in_progress));
    assert_eq!(
        json_ids(&answer["open_questions"]),
        row_ids(&view.open_questions)
    );
    assert_eq!(json_ids(&answer["unmet"]), row_ids(&view.unmet));
    let ready: Vec<String> = answer["ready"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["row"]["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        ready,
        view.ready
            .iter()
            .map(|item| item.row.id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        json_ids(&answer["resume"]["in_progress"]),
        view.resume
            .in_progress
            .iter()
            .map(|row| row.id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        answer["resume"]["open_questions"],
        view.resume.open_questions
    );
    assert_eq!(answer["resume"]["warnings"], view.resume.warnings);
    assert_eq!(
        answer["recent"].as_array().unwrap().len(),
        view.recent.len()
    );

    let scoped = json(&route(&repo, &get("/api/now", Some("scope=b")), "gy"));
    let view_b = gy_ledger::now(&repo, Some("b")).unwrap();
    assert_eq!(scoped["scope"], "b");
    assert_eq!(
        json_ids(&scoped["in_progress"]),
        row_ids(&view_b.in_progress)
    );
    assert_eq!(
        json_ids(&scoped["open_questions"]),
        row_ids(&view_b.open_questions)
    );
}

#[test]
fn now_json_carries_unwaited_questions() {
    let mut repo = ledger();
    QuestionAdd {
        scope: "a".to_string(),
        title: "who decides".to_string(),
        decider: "master".to_string(),
        options: vec!["x".to_string(), "y".to_string()],
        body: None,
    }
    .run(&mut repo)
    .unwrap();
    let answer = json(&route(&repo, &get("/api/now", None), "gy"));
    let open = answer["open_questions"].as_array().unwrap();
    assert_eq!(open.len(), 1);
    assert_eq!(open[0]["unwaited"], true);
    assert!(open[0]["created"].as_str().is_some());
    assert!(answer["in_progress"][0].get("unwaited").is_none());
}

/// One response's body as JSON.
fn json(res: &gy_serve::http::Response) -> serde_json::Value {
    serde_json::from_slice(&res.body).unwrap()
}

/// The id column of a row array in an answer.
fn json_ids(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect()
}

/// The ids of a view's rows, in their order.
fn row_ids(rows: &[gy_ledger::NodeRow]) -> Vec<String> {
    rows.iter().map(|row| row.id.clone()).collect()
}

#[test]
fn the_index_language_follows_accept_language() {
    let index = |lang: &str| {
        let headers = [("accept-language".to_string(), lang.to_string())];
        let res = route(&ledger(), &Request::new("GET", "/", None, &headers), "gy");
        String::from_utf8(res.body).unwrap()
    };
    assert!(index("ja,en;q=0.9").contains(r#"lang="ja""#));
    assert!(index("en-US").contains(r#"lang="en""#));
}

/// The name goes in last, so a name that looks like the other placeholder is
/// not replaced by it (n-07f0).
#[test]
fn the_title_name_is_not_replaced_again() {
    let res = route(&ledger(), &Request::new("GET", "/", None, &[]), "%LANG%");
    let html = String::from_utf8(res.body).unwrap();
    assert!(html.contains("<title>gy - %LANG%</title>"), "{html}");
}
