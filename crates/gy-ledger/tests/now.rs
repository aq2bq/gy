use gy_ledger::link::Link;
use gy_ledger::{
    Actor, ClosedBy, Closure, CriterionSatisfy, FormatVersion, MemoryStore, NeedClose, Node,
    NodeData, NodeId, NodeKind, Operation, QuestionClose, Relation, Repository, ReqApprove,
    ReqDone, RequirementState, handover, next, now,
};

fn id(kind: NodeKind, hash: &str) -> NodeId {
    NodeId::from_hash(kind, hash).unwrap()
}

fn q(hash: &str) -> String {
    id(NodeKind::Question, hash).to_string()
}
fn n(hash: &str) -> String {
    id(NodeKind::Need, hash).to_string()
}
fn c(hash: &str) -> String {
    id(NodeKind::Criterion, hash).to_string()
}
fn r(hash: &str) -> String {
    id(NodeKind::Requirement, hash).to_string()
}

/// One node as one write, so `created` is ours and the write count is exact.
fn put(repo: &mut Repository<MemoryStore>, node: &Node) {
    repo.transaction("seed", "test", |repo| repo.put(node))
        .unwrap();
}

/// A plain node of any kind, ready to put.
fn node(kind: NodeKind, hash: &str, scope: &str, created: &str) -> Node {
    let id = id(kind, hash);
    let built = match kind {
        NodeKind::Need => Node::need(id, scope, created, "a need"),
        NodeKind::Question => Node::question(id, scope, created, "a question"),
        NodeKind::Criterion => Node::criterion(id, scope, created, "a criterion"),
        NodeKind::Requirement => {
            Node::requirement(id, scope, created, "a requirement", RequirementState::Filed)
        }
        NodeKind::Decision => unreachable!("this ledger holds no decision"),
    };
    built.unwrap()
}

/// A question with a decider and two options, as one write.
fn ask(
    repo: &mut Repository<MemoryStore>,
    hash: &str,
    scope: &str,
    created: &str,
    decider: &str,
) -> NodeId {
    let id = id(NodeKind::Question, hash);
    let mut node = node(NodeKind::Question, hash, scope, created);
    if let NodeData::Question(data) = node.data_mut() {
        data.decider = Some(decider.to_string());
        data.options = vec!["x".to_string(), "y".to_string()];
    }
    put(repo, &node);
    id
}

fn need(repo: &mut Repository<MemoryStore>, hash: &str, scope: &str, created: &str) -> NodeId {
    let id = id(NodeKind::Need, hash);
    put(repo, &node(NodeKind::Need, hash, scope, created));
    id
}

fn criterion(repo: &mut Repository<MemoryStore>, hash: &str, scope: &str, created: &str) -> NodeId {
    let id = id(NodeKind::Criterion, hash);
    put(repo, &node(NodeKind::Criterion, hash, scope, created));
    id
}

fn requirement(
    repo: &mut Repository<MemoryStore>,
    hash: &str,
    scope: &str,
    created: &str,
) -> NodeId {
    let id = id(NodeKind::Requirement, hash);
    put(repo, &node(NodeKind::Requirement, hash, scope, created));
    id
}

/// Four questions, four needs, three criteria, and three requirements in two
/// scopes, as 21 writes.
fn ledger() -> Repository<MemoryStore> {
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string(), "b".to_string()]);
    ask(&mut repo, "0001", "a", "2026-09-01T00:00:00Z", "master");
    ask(&mut repo, "0002", "a", "2026-09-02T00:00:00Z", "マスター");
    ask(&mut repo, "0003", "a", "2026-09-03T00:00:00Z", "lead");
    let closed = ask(&mut repo, "0004", "b", "2026-09-04T00:00:00Z", "master");
    QuestionClose {
        id: closed,
        by: Closure::Fact,
        evidence: "x".into(),
        decision: None,
    }
    .run(&mut repo)
    .unwrap();
    need(&mut repo, "0005", "a", "2026-09-01T00:00:00Z");
    need(&mut repo, "0006", "b", "2026-09-02T00:00:00Z");
    let done = need(&mut repo, "0007", "b", "2026-09-03T00:00:00Z");
    let by_hand = need(&mut repo, "0008", "a", "2026-09-04T00:00:00Z");
    NeedClose {
        id: by_hand,
        by: ClosedBy::Fact,
        evidence: "x".into(),
    }
    .run(&mut repo)
    .unwrap();
    criterion(&mut repo, "0009", "a", "2026-09-01T00:00:00Z");
    criterion(&mut repo, "0010", "b", "2026-09-02T00:00:00Z");
    let met = criterion(&mut repo, "0011", "a", "2026-09-03T00:00:00Z");
    CriterionSatisfy {
        id: met,
        evidence: "x".into(),
        revoke: false,
    }
    .run(&mut repo)
    .unwrap();
    requirement(&mut repo, "0012", "b", "2026-09-01T00:00:00Z");
    let approved = requirement(&mut repo, "0013", "a", "2026-09-02T00:00:00Z");
    let shipped = requirement(&mut repo, "0014", "b", "2026-09-03T00:00:00Z");
    for requirement in [&approved, &shipped] {
        ReqApprove {
            id: requirement.clone(),
            design: "x".into(),
            heard_by: "master".into(),
            evidence: "x".into(),
        }
        .run(&mut repo)
        .unwrap();
    }
    ReqDone {
        id: shipped.clone(),
        evidence: "x".into(),
    }
    .run(&mut repo)
    .unwrap();
    Link {
        from: done,
        relation: Relation::FiledAs,
        to: shipped,
        mark: None,
        remove: false,
    }
    .run(&mut repo)
    .unwrap();
    repo
}

fn ids(rows: &[gy_ledger::NodeRow]) -> Vec<String> {
    rows.iter().map(|row| row.id.clone()).collect()
}

#[test]
fn now_reads_the_whole_ledger() {
    let repo = ledger();
    let view = now(&repo, None).unwrap();
    assert_eq!(view.scope, None);
    assert_eq!(view.seq, 21);
    assert!(view.at > 0);

    // The master's queue: the two questions that name the master, oldest
    // first, then the filed requirement.
    assert_eq!(view.waiting.len(), 3);
    let waiting: Vec<String> = view
        .waiting
        .iter()
        .map(|item| match item {
            gy_ledger::Waiting::Question {
                row,
                decider,
                options,
            } => {
                assert_eq!(options.len(), 2);
                format!("{} {decider}", row.id)
            }
            gy_ledger::Waiting::Requirement { row, .. } => row.id.clone(),
        })
        .collect();
    assert_eq!(
        waiting,
        [
            format!("{} master", q("0001")),
            format!("{} マスター", q("0002")),
            r("0012")
        ]
    );

    // Open needs, newest first; the done and hand-closed needs stay out.
    assert_eq!(ids(&view.in_progress), [n("0006"), n("0005")]);
    assert_eq!(ids(&view.open_questions), [q("0003")]);
    assert_eq!(ids(&view.unmet), [c("0010"), c("0009")]);

    // The last fourteen writes, newest first.
    assert_eq!(view.recent.len(), 14);
    assert_eq!(view.recent[0].seq, view.seq);
    assert!(
        view.recent.windows(2).all(|pair| pair[0].seq > pair[1].seq),
        "recent is not newest first"
    );
}

#[test]
fn now_filters_by_scope() {
    let repo = ledger();
    let all = now(&repo, None).unwrap();
    let b = now(&repo, Some("b")).unwrap();
    assert_eq!(b.scope.as_deref(), Some("b"));
    assert_eq!(b.waiting.len(), 1);
    assert!(matches!(
        b.waiting[0],
        gy_ledger::Waiting::Requirement { .. }
    ));
    assert_eq!(ids(&b.in_progress), [n("0006")]);
    assert!(b.open_questions.is_empty());
    assert_eq!(ids(&b.unmet), [c("0010")]);

    // Only writes to scope b: every recent entry names one of its nodes.
    let in_b = [
        q("0004"),
        n("0006"),
        n("0007"),
        c("0010"),
        r("0012"),
        r("0014"),
    ];
    assert_eq!(b.recent.len(), 10);
    assert!(b.recent.len() < all.recent.len());
    assert!(
        b.recent.iter().all(|entry| in_b.contains(&entry.node)),
        "a recent entry is outside scope b"
    );
}

#[test]
fn now_reports_ready_and_resume_from_the_other_views() {
    let repo = ledger();
    let view = now(&repo, None).unwrap();

    // ready is next, row for row and count for count.
    let ready = next(&repo, None).unwrap();
    assert_eq!(view.ready.len(), ready.len());
    for (mine, theirs) in view.ready.iter().zip(&ready) {
        assert_eq!(mine.row.id, theirs.id);
        assert_eq!(
            (mine.satisfied, mine.targets),
            (theirs.satisfied, theirs.targets)
        );
    }

    // resume is handover, plus the last write.
    let session = handover(&repo, None).unwrap();
    let mine: Vec<&String> = view.resume.in_progress.iter().map(|row| &row.id).collect();
    let theirs: Vec<&String> = session.in_progress.iter().map(|row| &row.id).collect();
    assert_eq!(mine, theirs);
    assert_eq!(view.resume.open_questions, session.open_questions);
    assert_eq!(
        view.resume.warnings,
        session
            .warnings
            .iter()
            .map(|warning| warning.count)
            .sum::<usize>()
    );
    assert_eq!(view.resume.last.as_ref().unwrap().seq, view.seq);

    // The scope narrows both, the same way the views do.
    let b = now(&repo, Some("b")).unwrap();
    assert_eq!(b.ready.len(), next(&repo, Some("b")).unwrap().len());
    assert_eq!(
        b.resume.in_progress.len(),
        handover(&repo, Some("b")).unwrap().in_progress.len()
    );
}

#[test]
fn now_marks_questions_nobody_waits_on() {
    let repo = ledger();
    let view = now(&repo, None).unwrap();
    assert_eq!(view.open_questions.len(), 1);
    assert!(view.open_questions[0].unwaited);
    assert_eq!(view.open_questions[0].created, "2026-09-03T00:00:00Z");
    let rows = view
        .waiting
        .iter()
        .filter_map(|item| match item {
            gy_ledger::Waiting::Question { row, .. } => Some(row.unwaited),
            gy_ledger::Waiting::Requirement { .. } => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(rows, [true, true]);
    assert!(view.in_progress.iter().all(|row| !row.unwaited));
    assert!(view.ready.iter().all(|item| !item.row.unwaited));
}
