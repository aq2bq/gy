use chrono::NaiveDate;
use gy_ledger::{
    Actor, CriterionSatisfy, Edit, FormatVersion, Link, MemoryStore, Node, NodeId, NodeKind,
    Operation, Relation, Repository, RequirementState, Store, handover_on, next_on, now, show,
};

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}
fn satisfy(ac: &Node) -> CriterionSatisfy {
    CriterionSatisfy {
        id: ac.id().clone(),
        evidence: "measured early".into(),
        revoke: false,
    }
}
fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
}
const CREATED: &str = "2026-10-01T00:00:00Z";
fn repository() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("due-test").unwrap(),
    ))
}
fn criterion(hash: &str, scope: &str, due: &str) -> Node {
    let mut node = Node::criterion(
        id(NodeKind::Criterion, hash),
        scope,
        CREATED,
        &format!("measure {hash}"),
    )
    .unwrap();
    if !due.is_empty() {
        node.set_free("due", due);
        node.set_free("responsible", "ops-lead");
        node.set_free("then", "measure the result");
    }
    node
}
fn seed(repo: &mut Repository<MemoryStore>, nodes: &[Node]) {
    repo.transaction("seed", "test", |repo| {
        for node in nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();
}
fn edit(node: &Node, set: &[(&str, &str)]) -> Edit {
    Edit {
        id: node.id().clone(),
        reason: "date the wait".into(),
        title: None,
        body: None,
        set: set
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        append: Vec::new(),
    }
}

#[test]
fn edit_and_show_named_attributes_without_full() {
    let mut repo = repository();
    let ac = criterion("0001", "a", "");
    seed(&mut repo, std::slice::from_ref(&ac));
    let attrs = [
        ("then", "decide"),
        ("responsible", "ops-lead"),
        ("due", "2028-02-29"),
    ];
    edit(&ac, &attrs).run(&mut repo).unwrap();
    let shown = show(&repo, &[ac.id().to_string()], false).unwrap();
    for line in ["due: 2028-02-29", "responsible: ops-lead", "then: decide"] {
        assert!(shown[0].to_string().contains(line));
    }
    assert_eq!(
        serde_json::to_value(&shown).unwrap()[0]["attributes"]["due"],
        "2028-02-29"
    );
    edit(&ac, &[("due", ""), ("responsible", ""), ("then", "")])
        .run(&mut repo)
        .unwrap();
    let cleared = repo.get(ac.id()).unwrap().unwrap();
    assert!(cleared.free_attributes().is_empty());
}

#[test]
fn invalid_dates_and_dateless_waits_are_atomic_refusals() {
    let mut repo = repository();
    let ac = criterion("0001", "a", "");
    seed(&mut repo, std::slice::from_ref(&ac));
    let bad = [
        "tomorrow",
        "2026-02-29",
        "2026-13-01",
        "2026-10-32",
        "2026-1-01",
        " 2026-10-01",
    ];
    for value in bad {
        let error = edit(&ac, &[("due", value)])
            .run(&mut repo)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("YYYY-MM-DD") && error.contains("next: gy edit ac-0001"),
            "{error}"
        );
    }
    for key in ["responsible", "then"] {
        let error = edit(&ac, &[(key, "waiting")])
            .run(&mut repo)
            .unwrap_err()
            .to_string();
        assert!(error.contains("date when you will decide it"), "{error}");
    }
    assert_eq!(repo.store().history().len(), 1);
    assert_eq!(repo.get(ac.id()).unwrap().unwrap(), ac);
    edit(&ac, &[("due", "2026-10-26"), ("then", "measure")])
        .run(&mut repo)
        .unwrap();
    assert!(edit(&ac, &[("due", "")]).run(&mut repo).is_err());
    let mut append = edit(&ac, &[]);
    append.append = vec![("due".into(), "2026-10-27".into())];
    assert!(append.run(&mut repo).is_err());
    assert_eq!(
        repo.get(ac.id()).unwrap().unwrap().free("due").unwrap(),
        "2026-10-26"
    );
}

#[test]
fn other_kinds_keep_free_attribute_semantics() {
    let mut repo = repository();
    let need = Node::need(id(NodeKind::Need, "0001"), "a", CREATED, "need").unwrap();
    seed(&mut repo, std::slice::from_ref(&need));
    let attrs = [
        ("due", "whenever"),
        ("then", "later"),
        ("responsible", "ops"),
    ];
    edit(&need, &attrs).run(&mut repo).unwrap();
    edit(&need, &[("due", "")]).run(&mut repo).unwrap();
}

fn fixture() -> (Repository<MemoryStore>, Node) {
    let mut repo = repository();
    let nodes = [
        criterion("0001", "a", "2026-10-25"),
        criterion("0002", "a", "2026-10-26"),
        criterion("0003", "a", "2026-10-27"),
        criterion("0004", "b", "2026-10-24"),
        criterion("0005", "a", "2099-10-28"),
    ];
    let mut need = Node::need(id(NodeKind::Need, "0001"), "a", CREATED, "need").unwrap();
    let mut req = Node::requirement(
        id(NodeKind::Requirement, "0001"),
        "a",
        CREATED,
        "request",
        RequirementState::Approved,
    )
    .unwrap();
    for ac in &nodes {
        need.link(Link::new(need.id().clone(), Relation::Targets, ac.id().clone()).unwrap());
        req.link(Link::new(req.id().clone(), Relation::Targets, ac.id().clone()).unwrap());
    }
    seed(&mut repo, &nodes);
    seed(&mut repo, &[need, req]);
    (repo, nodes[4].clone())
}

#[test]
fn satisfy_before_due_keeps_the_existing_gate_and_hides_satisfied_waits() {
    let (mut repo, ac) = fixture();
    satisfy(&ac).run(&mut repo).unwrap();
    let h = handover_on(&repo, None, date("2100-01-01")).unwrap();
    assert!(!h.to_string().contains("ac-0005"));
    let rows = next_on(&repo, None, date("2100-01-01")).unwrap();
    assert!(!serde_json::to_string(&rows).unwrap().contains("ac-0005"));
    let uncovered = criterion("0006", "a", "2099-10-28");
    seed(&mut repo, std::slice::from_ref(&uncovered));
    let error = satisfy(&uncovered).run(&mut repo).unwrap_err();
    assert!(error.to_string().contains("approved requirement"));
}

#[test]
fn handover_has_due_details_and_only_future_count_and_earliest_date() {
    let (repo, _) = fixture();
    let h = handover_on(&repo, Some("a"), date("2026-10-26")).unwrap();
    assert_eq!(
        h.due.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        ["ac-0001", "ac-0002"]
    );
    let text = h.to_string();
    assert!(
        text.contains(
            "ac-0002 due 2026-10-26 measure 0002\n  responsible: ops-lead\n  then: measure"
        )
    );
    assert!(text.contains("waiting until a date: 2 (earliest 2026-10-27)"));
    assert!(!text.contains("next:"));
    for hidden in ["ac-0003", "ac-0004", "ac-0005", "2099-10-28"] {
        assert!(!text.contains(hidden));
    }
    let json = serde_json::to_value(&h).unwrap();
    assert_eq!(json["due"][1]["responsible"], "ops-lead");
    assert_eq!(json["due"][1]["then"], "measure the result");
    assert_eq!(json["waiting_until"]["count"], 2);
    assert_eq!(json["waiting_until"]["earliest"], "2026-10-27");
    let all = handover_on(&repo, None, date("2026-10-26")).unwrap();
    let absent = handover_on(&repo, Some("missing"), date("2026-10-26")).unwrap();
    let tomorrow = handover_on(&repo, Some("a"), date("2026-10-27")).unwrap();
    assert_eq!(all.due.len(), 3);
    assert!(absent.due.is_empty());
    assert_eq!(tomorrow.due.len(), 3);
}

#[test]
fn next_links_waits_with_status_and_scope_without_changing_now() {
    let (repo, _) = fixture();
    let rows = next_on(&repo, Some("a"), date("2026-10-26")).unwrap();
    assert_eq!(rows.len(), 1);
    let text = rows[0].to_string();
    assert!(text.contains("  ac-0002 due 2026-10-26 (due)"));
    assert!(text.contains("  ac-0003 due 2026-10-27 (waiting)"));
    assert!(!text.contains("ac-0004"));
    assert_eq!(rows[0].criteria.len(), 4);
    let json = serde_json::to_value(&rows).unwrap();
    assert_eq!(json[0]["criteria"][1]["status"], "due");
    assert_eq!(json[0]["criteria"][2]["status"], "waiting");
    assert!(
        next_on(&repo, Some("b"), date("2026-10-26"))
            .unwrap()
            .is_empty()
    );
    let mut without_dates = repository();
    let mut nodes = repo.all().unwrap();
    for node in &mut nodes {
        for key in ["due", "responsible", "then"] {
            node.remove_free(key);
        }
    }
    seed(&mut without_dates, &nodes);
    let mut before = serde_json::to_value(now(&without_dates, None).unwrap()).unwrap();
    let mut after = serde_json::to_value(now(&repo, None).unwrap()).unwrap();
    // Seeding uses different transactions; compare every projected field except log metadata.
    for value in [&mut before, &mut after] {
        for key in ["seq", "at", "recent"] {
            value.as_object_mut().unwrap().remove(key);
        }
        value["resume"].as_object_mut().unwrap().remove("last");
    }
    assert_eq!(before, after);
}
