//! Running one generated write against the ledger (n-99c6, n-a006): resolve
//! the operands from the nodes created so far, then call the operation.
use super::{ALLOWED, Kind, Op, is_approved, is_closed_need};
use gy_ledger::{
    ClosedBy, CriterionAdd, CriterionSatisfy, Decide, DecisionScope, Edit, MemoryStore, NeedAdd,
    NeedClose, Node, NodeId, NodeKind, Operation, QuestionAdd, Relation, Repository, ReqAdd,
    ReqApprove, ReqCancel, ReqDone, Undo, link,
};

/// Run one write against the nodes it read. `None` skips a write whose operand
/// the ledger does not hold; the error is the refusal the property reads.
pub(super) fn run(
    op: Op,
    created: &[NodeId],
    repo: &mut Repository<MemoryStore>,
) -> Option<gy_ledger::Result<Option<NodeId>>> {
    let result = match op.kind {
        Kind::CriterionAdd => CriterionAdd {
            scope: "a".into(),
            title: "ac".into(),
            body: Some("body".into()),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::QuestionAdd => QuestionAdd {
            scope: "a".into(),
            title: "q".into(),
            decider: "m".into(),
            options: vec!["a".into(), "b".into()],
            body: Some("body".into()),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::NeedAdd => NeedAdd {
            scope: "a".into(),
            title: "n".into(),
            targets: vec![
                pick(created, repo, NodeKind::Criterion, op.a)?,
                pick(created, repo, NodeKind::Criterion, op.b)?,
            ],
            spawned_by: if op.flag {
                Some(pick(created, repo, NodeKind::Decision, op.b)?)
            } else {
                None
            },
            body: Some("body".into()),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::ReqAdd => ReqAdd {
            scope: "a".into(),
            title: "r".into(),
            needs: vec![
                pick(created, repo, NodeKind::Need, op.a)?,
                need(created, repo, op.b, op.flag)?,
            ],
            relies_on: vec![],
            targets: vec![pick(created, repo, NodeKind::Criterion, op.rel)?],
            reference: None,
            body: Some("body".into()),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Decide => Decide {
            scope: "a".into(),
            title: "d".into(),
            decision_scope: DecisionScope::recorded("conditions").ok()?,
            body: Some("body m1".into()),
            source: None,
            closes: vec![pick(created, repo, NodeKind::Question, op.a)?],
            relates: vec![],
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Link => {
            let (relation, from_kind, to_kind) = ALLOWED[op.rel as usize % ALLOWED.len()];
            let from = if op.flag && relation == Relation::FiledAs {
                closed_need(created, repo, op.a)?
            } else {
                pick(created, repo, from_kind, op.a)?
            };
            link::Link {
                from,
                relation,
                to: pick(created, repo, to_kind, op.b)?,
                mark: mark(op.mark),
                remove: false,
            }
            .run(repo)
            .map(|outcome| outcome.id)
        }
        Kind::Stray => link::Link {
            from: pick(created, repo, NodeKind::ALL[op.a as usize % 5], op.a)?,
            relation: Relation::ALL[op.rel as usize % 12],
            to: pick(created, repo, NodeKind::ALL[op.b as usize % 5], op.b)?,
            mark: mark(op.mark),
            remove: false,
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Repeat => {
            let (from, relation, to, mark) = nth_edge(created, repo, op.a)?;
            link::Link {
                from,
                relation,
                to,
                mark,
                remove: false,
            }
            .run(repo)
            .map(|outcome| outcome.id)
        }
        Kind::Satisfy => CriterionSatisfy {
            id: pick(created, repo, NodeKind::Criterion, op.a)?,
            evidence: if op.flag { String::new() } else { "e".into() },
            revoke: false,
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::NeedClose => NeedClose {
            id: pick(created, repo, NodeKind::Need, op.a)?,
            by: ClosedBy::Fact,
            evidence: "e".into(),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Approve => ReqApprove {
            id: pick(created, repo, NodeKind::Requirement, op.a)?,
            design: "d".into(),
            heard_by: "h".into(),
            evidence: "e".into(),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Done => ReqDone {
            id: pick(created, repo, NodeKind::Requirement, op.a)?,
            evidence: "e".into(),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Cancel => ReqCancel {
            id: pick(created, repo, NodeKind::Requirement, op.a)?,
            reason: "r".into(),
            source: "s".into(),
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Edit => Edit {
            id: approved_requirement(created, repo, op.a)?,
            reason: "e".into(),
            title: Some("t".into()),
            body: op.flag.then(|| "b".into()),
            set: vec![],
            append: vec![],
        }
        .run(repo)
        .map(|outcome| outcome.id),
        Kind::Undo => Undo { reason: "u".into() }
            .run(repo)
            .map(|outcome| outcome.id),
    };
    Some(result)
}

/// Whether the write creates a node (an id to add to the creation-order list).
pub(super) fn creates(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::CriterionAdd | Kind::QuestionAdd | Kind::NeedAdd | Kind::ReqAdd | Kind::Decide
    )
}

/// The `n`-th live node of a kind, in creation order. `n` of 8 or more is an
/// id the ledger does not hold (E4).
fn pick(
    created: &[NodeId],
    repo: &Repository<MemoryStore>,
    kind: NodeKind,
    n: u8,
) -> Option<NodeId> {
    if n >= 8 {
        return NodeId::from_hash(kind, format!("zz{n}")).ok();
    }
    nth_matching(created, repo, kind, n, |_| true)
}

/// A need that is filed-as a closed requirement (E8), in creation order.
fn closed_need(created: &[NodeId], repo: &Repository<MemoryStore>, n: u8) -> Option<NodeId> {
    if n >= 8 {
        return NodeId::from_hash(NodeKind::Need, format!("zz{n}")).ok();
    }
    nth_matching(created, repo, NodeKind::Need, n, is_closed_need)
}

/// An approved requirement (R5), in creation order.
fn approved_requirement(
    created: &[NodeId],
    repo: &Repository<MemoryStore>,
    n: u8,
) -> Option<NodeId> {
    if n >= 8 {
        return NodeId::from_hash(NodeKind::Requirement, format!("zz{n}")).ok();
    }
    nth_matching(created, repo, NodeKind::Requirement, n, is_approved)
}

fn need(created: &[NodeId], repo: &Repository<MemoryStore>, n: u8, closed: bool) -> Option<NodeId> {
    if closed {
        closed_need(created, repo, n)
    } else {
        pick(created, repo, NodeKind::Need, n)
    }
}

fn nth_matching(
    created: &[NodeId],
    repo: &Repository<MemoryStore>,
    kind: NodeKind,
    n: u8,
    keep: fn(&Node) -> bool,
) -> Option<NodeId> {
    let alive: Vec<&NodeId> = created
        .iter()
        .filter(|id| {
            id.kind() == kind
                && repo
                    .get(id)
                    .is_ok_and(|node| node.is_some_and(|node| keep(&node)))
        })
        .collect();
    nth(&alive, n)
}

fn nth(alive: &[&NodeId], n: u8) -> Option<NodeId> {
    if alive.is_empty() {
        return None;
    }
    Some(alive[n as usize % alive.len()].clone())
}

fn mark(n: u8) -> Option<String> {
    (n != 0).then(|| format!("m{n}"))
}

/// An existing forward edge, in creation order, to repeat (E3).
fn nth_edge(
    created: &[NodeId],
    repo: &Repository<MemoryStore>,
    n: u8,
) -> Option<(NodeId, Relation, NodeId, Option<String>)> {
    let mut edges = Vec::new();
    for id in created {
        let Some(node) = repo.get(id).ok().flatten() else {
            continue;
        };
        for edge in node.links().iter().filter(|edge| !edge.reversed) {
            edges.push((
                edge.from.clone(),
                edge.label,
                edge.to.clone(),
                edge.mark.clone(),
            ));
        }
    }
    if edges.is_empty() {
        return None;
    }
    Some(edges[n as usize % edges.len()].clone())
}
