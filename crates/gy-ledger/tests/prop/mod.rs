//! The shared generator and replay for the invariant property tests (n-99c6,
//! n-a006). A plan is one random sequence of writes; replay records what every
//! step saw and did. P1a, P1b and P3 read this module with `mod prop;`. The
//! rule predicates live outside this module (`rules.rs`), included by the test
//! that reads them, so a reader that does not judge E1-E4 does not carry them
//! (dead code would otherwise fail the build).
use gy_ledger::{
    Actor, FormatVersion, MemoryStore, Node, NodeData, NodeId, NodeKind, Relation, Repository,
    RequirementState, Store,
};
use proptest::prelude::*;
use std::collections::BTreeSet;

mod step;
use step::{creates, run};

pub const CASES: u32 = 16;
pub const MAX_OPS: usize = 24;

/// The (from kind, relation, to kind) pairs of `model/links.rs:ALLOWED`, read by
/// the generator to pick a link a write would make, and by the E1 predicate.
pub const ALLOWED: &[(Relation, NodeKind, NodeKind)] = &[
    (Relation::Closes, NodeKind::Question, NodeKind::Decision),
    (Relation::Narrows, NodeKind::Decision, NodeKind::Decision),
    (Relation::Widens, NodeKind::Decision, NodeKind::Decision),
    (Relation::Supersedes, NodeKind::Decision, NodeKind::Decision),
    (Relation::Completes, NodeKind::Decision, NodeKind::Decision),
    (Relation::Targets, NodeKind::Need, NodeKind::Criterion),
    (
        Relation::Targets,
        NodeKind::Requirement,
        NodeKind::Criterion,
    ),
    (Relation::SpawnedBy, NodeKind::Need, NodeKind::Decision),
    (Relation::FiledAs, NodeKind::Need, NodeKind::Requirement),
    (Relation::DependsOn, NodeKind::Need, NodeKind::Need),
    (
        Relation::ReliesOn,
        NodeKind::Requirement,
        NodeKind::Decision,
    ),
    (Relation::Raised, NodeKind::Requirement, NodeKind::Question),
    (Relation::WaitsOn, NodeKind::Need, NodeKind::Question),
    (Relation::WaitsOn, NodeKind::Need, NodeKind::Requirement),
];

pub fn is_approved(node: &Node) -> bool {
    node.state() == Some(RequirementState::Approved)
}

pub fn is_closed_need(node: &Node) -> bool {
    matches!(node.data(), NodeData::Need(data) if data.closed.is_some())
}

/// A rule that can refuse a write. E2 and T5 are state invariants: no write is
/// refused for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    E1,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
    R5,
    C1,
    C2,
}

/// A write the generator can make. `a` and `b` pick an operand by position in
/// the order the nodes were created (8 or more means an id that does not
/// exist, E4); `rel` indexes `rules::ALLOWED` for `Link` and `Relation::ALL`
/// for `Stray`; `mark` 0 is no mark; `flag` selects a variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    CriterionAdd,
    QuestionAdd,
    NeedAdd,
    ReqAdd,
    Decide,
    Link,
    Stray,
    Repeat,
    Satisfy,
    NeedClose,
    Approve,
    Done,
    Cancel,
    Edit,
    Undo,
}

#[derive(Clone, Copy, Debug)]
pub struct Op {
    pub kind: Kind,
    pub a: u8,
    pub b: u8,
    pub rel: u8,
    pub mark: u8,
    pub flag: bool,
}

/// The ledger at one instant: its log length and every node.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub seq: u64,
    pub nodes: Vec<Node>,
}

/// One step of a replay.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub before: State,
    pub after: State,
    pub kind: Kind,
    pub accepted: bool,
    pub hits: BTreeSet<Rule>,
}

/// A random plan: the setup the rules need, then the writes to try.
pub fn plan() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(op(), 4..=MAX_OPS).prop_map(|mut tail| {
        let mut plan = prologue();
        plan.append(&mut tail);
        plan
    })
}

/// The state the rules need: criteria, questions, needs, a filed and approved
/// requirement, a closed need and a question closed by decision, so an
/// operation has targets, an edge to repeat, and the states E5–E8, R5, C1 and
/// C2 judge. P3's derived judgements need more: a shipped requirement whose
/// need still has an unmet criterion (Nd3's d-85c6 branch), a need whose only
/// requirement was cancelled (d-bf90), a narrowed passage and a superseded
/// decision (D2), and a depends-on and a waits-on edge (Nd5).
fn prologue() -> Vec<Op> {
    let op = |kind, a, b| Op {
        kind,
        a,
        b,
        rel: 0,
        mark: 0,
        flag: false,
    };
    let link = |rel, a, b, mark| Op {
        kind: Kind::Link,
        a,
        b,
        rel,
        mark,
        flag: false,
    };
    vec![
        op(Kind::CriterionAdd, 0, 0),
        op(Kind::CriterionAdd, 0, 0),
        op(Kind::CriterionAdd, 0, 0),
        op(Kind::QuestionAdd, 0, 0),
        op(Kind::QuestionAdd, 0, 0),
        op(Kind::NeedAdd, 0, 1),
        op(Kind::NeedAdd, 2, 0),
        op(Kind::NeedAdd, 1, 2),
        op(Kind::ReqAdd, 0, 1),
        op(Kind::Approve, 0, 0),
        // An edit of the approved R0 is refused (R5), so the random tail need
        // not reach a frozen requirement for the rule to have a hit.
        op(Kind::Edit, 0, 0),
        op(Kind::NeedClose, 1, 0),
        op(Kind::Decide, 0, 0),
        // c0 is covered by approved R0; satisfy it and ship R0, so N0 (targets
        // c0, c1) has a done requirement and an unmet criterion. 1.2.2 read
        // that as done (n-f60a); d-85c6 does not.
        op(Kind::Satisfy, 0, 0),
        op(Kind::Done, 0, 0),
        // A second requirement filed for N2 and N0 is cancelled, so N2 has
        // only a cancelled requirement and asks for a new one (d-bf90).
        Op {
            kind: Kind::ReqAdd,
            a: 2,
            b: 0,
            rel: 1,
            mark: 0,
            flag: false,
        },
        op(Kind::Cancel, 1, 0),
        // A second decision supersedes D0 and narrows the passage `m1` its
        // body carries, for D2's retraction.
        op(Kind::Decide, 1, 0),
        link(3, 1, 0, 0),
        link(1, 1, 0, 1),
        // N2 depends on the closed N1, so it is ready; N0 waits on the open q1,
        // so it is not (Nd5).
        link(9, 2, 1, 0),
        link(12, 0, 1, 0),
        // A need that genuinely reaches done (Nd4): every requirement it is
        // filed as shipped and every targeted criterion satisfied.
        op(Kind::CriterionAdd, 0, 0),
        op(Kind::CriterionAdd, 0, 0),
        op(Kind::NeedAdd, 3, 4),
        Op {
            kind: Kind::ReqAdd,
            a: 3,
            b: 0,
            rel: 3,
            mark: 0,
            flag: false,
        },
        op(Kind::Approve, 2, 0),
        op(Kind::Satisfy, 3, 0),
        Op {
            kind: Kind::ReqAdd,
            a: 3,
            b: 0,
            rel: 4,
            mark: 0,
            flag: false,
        },
        op(Kind::Approve, 3, 0),
        op(Kind::Satisfy, 4, 0),
        op(Kind::Done, 2, 0),
        op(Kind::Done, 3, 0),
        // N4 is otherwise ready but depends on the open N2 (Nd5's depends-on
        // branch), and N5 is otherwise ready but waits on the cancelled R1
        // (Nd5's waits-on branch).
        op(Kind::NeedAdd, 0, 1),
        link(9, 4, 2, 0),
        op(Kind::NeedAdd, 0, 1),
        link(13, 5, 1, 0),
        // c5 is covered by the shipped R4 and left unsatisfied: a done
        // requirement covers a criterion the way an approved one does (C3).
        op(Kind::CriterionAdd, 0, 0),
        op(Kind::NeedAdd, 5, 0),
        Op {
            kind: Kind::ReqAdd,
            a: 6,
            b: 0,
            rel: 5,
            mark: 0,
            flag: false,
        },
        op(Kind::Approve, 4, 0),
        op(Kind::Done, 4, 0),
        // Refused writes, one per rule the random tail would otherwise have to
        // reach, so no hit count depends on the number of cases: E1 a link
        // whose pair is not allowed, E3 a second copy of an existing edge, E4
        // an operand the ledger does not hold, E6 a closes edge made by link,
        // E8 a closed need taking a new filed-as requirement.
        Op {
            kind: Kind::Stray,
            a: 4,
            b: 4,
            rel: 0,
            mark: 0,
            flag: false,
        },
        op(Kind::Repeat, 0, 0),
        op(Kind::NeedClose, 8, 0),
        link(0, 0, 0, 0),
        Op {
            kind: Kind::Link,
            a: 0,
            b: 0,
            rel: 8,
            mark: 0,
            flag: true,
        },
        // A criterion no need bears, left unsatisfied: the bearer-less gap the
        // advice reports rather than an orphan (C4), so the M6 direction is
        // exercised without the random tail.
        op(Kind::CriterionAdd, 0, 0),
    ]
}

fn op() -> impl Strategy<Value = Op> {
    let kinds = prop_oneof![
        2 => Just(Kind::CriterionAdd),
        2 => Just(Kind::QuestionAdd),
        2 => Just(Kind::NeedAdd),
        2 => Just(Kind::ReqAdd),
        2 => Just(Kind::Decide),
        4 => Just(Kind::Link),
        4 => Just(Kind::Stray),
        4 => Just(Kind::Repeat),
        2 => Just(Kind::Satisfy),
        2 => Just(Kind::NeedClose),
        2 => Just(Kind::Approve),
        2 => Just(Kind::Done),
        2 => Just(Kind::Cancel),
        2 => Just(Kind::Edit),
        2 => Just(Kind::Undo),
    ];
    let fields = (
        kinds,
        0u8..16,
        any::<bool>(),
        0u8..16,
        0u8..14,
        0u8..4,
        any::<bool>(),
    );
    fields.prop_map(|(kind, a, dup, b, rel, mark, flag)| Op {
        kind,
        a,
        // Half the time `b` is `a`, so a duplicated edge (E3) is easy to reach.
        b: if dup { a } else { b },
        rel,
        mark,
        flag,
    })
}

/// Replay the plan, handing each step and the repository in its after state to
/// `visit`. A write whose operands the ledger does not hold is skipped (it
/// leaves no step). Node ids are kept in the order they were created, so the
/// same plan always picks the same nodes (the generator is deterministic). A
/// visitor that reports an error stops the replay.
pub fn replay_visit(
    plan: &[Op],
    mut visit: impl FnMut(&Step, &Repository<MemoryStore>) -> Result<(), TestCaseError>,
) -> Result<(), TestCaseError> {
    let mut repo = repo();
    let mut created: Vec<NodeId> = Vec::new();
    for op in plan {
        let before = snapshot(&repo);
        let hits = hits(op);
        let Some(result) = run(*op, &created, &mut repo) else {
            continue;
        };
        if let Ok(Some(id)) = &result {
            if creates(op.kind) {
                created.push(id.clone());
            }
        }
        let step = Step {
            before,
            after: snapshot(&repo),
            kind: op.kind,
            accepted: result.is_ok(),
            hits,
        };
        visit(&step, &repo)?;
    }
    Ok(())
}

fn repo() -> Repository<MemoryStore> {
    Repository::new(MemoryStore::with_actor(
        FormatVersion::CURRENT,
        Actor::new("piko").unwrap(),
    ))
}

fn snapshot(repo: &Repository<MemoryStore>) -> State {
    State {
        seq: repo.store().history().len() as u64,
        nodes: repo.all().unwrap(),
    }
}

/// The rules a write is in the domain of, from its kind and relation alone:
/// enough to see that an item was exercised (a count of zero fails the item).
pub fn hits(op: &Op) -> BTreeSet<Rule> {
    let mut set = BTreeSet::new();
    match op.kind {
        Kind::Stray => {
            set.insert(Rule::E1);
        }
        Kind::Repeat => {
            set.insert(Rule::E3);
        }
        Kind::Satisfy => {
            set.insert(Rule::C1);
            set.insert(Rule::C2);
        }
        Kind::Edit => {
            set.insert(Rule::R5);
        }
        Kind::ReqAdd if op.flag => {
            set.insert(Rule::E8);
        }
        _ => {}
    }
    if op.a >= 8 || op.b >= 8 {
        set.insert(Rule::E4);
    }
    match relation_of(op) {
        Some(Relation::Narrows | Relation::Supersedes) => {
            set.insert(Rule::E5);
            set.insert(Rule::E7);
        }
        Some(Relation::Closes) => {
            set.insert(Rule::E6);
        }
        Some(Relation::DependsOn | Relation::Widens | Relation::Completes) => {
            set.insert(Rule::E7);
        }
        Some(Relation::FiledAs) => {
            set.insert(Rule::E8);
        }
        _ => {}
    }
    set
}

/// The relation an edge write would use, or `None` for a write that makes no
/// edge (a `Repeat`'s edge is resolved when it runs).
fn relation_of(op: &Op) -> Option<Relation> {
    match op.kind {
        Kind::Link => Some(ALLOWED[op.rel as usize % ALLOWED.len()].0),
        Kind::Stray => Some(Relation::ALL[op.rel as usize % 12]),
        _ => None,
    }
}
