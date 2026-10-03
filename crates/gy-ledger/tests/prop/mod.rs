//! The shared generator and replay for the invariant property tests (n-99c6,
//! n-a006). A plan is one random sequence of writes; replay records what every
//! step saw and did. P1a, P1b, P2 and P3 read this module with `mod prop;`.
use gy_ledger::{Actor, FormatVersion, MemoryStore, Node, NodeId, Relation, Repository, Store};
use proptest::prelude::*;
use std::collections::BTreeSet;

pub mod rules;
mod step;
use step::{creates, run};

pub const CASES: u32 = 96;
pub const MAX_OPS: usize = 24;

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

#[derive(Clone, Debug, Default)]
pub struct Trace {
    pub steps: Vec<Step>,
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
/// C2 judge.
fn prologue() -> Vec<Op> {
    let op = |kind, a, b| Op {
        kind,
        a,
        b,
        rel: 0,
        mark: 0,
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
        op(Kind::NeedClose, 1, 0),
        op(Kind::Decide, 0, 0),
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

/// Replay the plan, recording what each step saw and did. A write whose
/// operands the ledger does not hold is skipped (it leaves no step). Node ids
/// are kept in the order they were created, so the same plan always picks the
/// same nodes (the generator is deterministic).
pub fn replay(plan: &[Op]) -> Trace {
    let mut repo = repo();
    let mut created: Vec<NodeId> = Vec::new();
    let mut steps = Vec::new();
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
        steps.push(Step {
            before,
            after: snapshot(&repo),
            kind: op.kind,
            accepted: result.is_ok(),
            hits,
        });
    }
    Trace { steps }
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
        Kind::Link => Some(rules::ALLOWED[op.rel as usize % rules::ALLOWED.len()].0),
        Kind::Stray => Some(Relation::ALL[op.rel as usize % 12]),
        _ => None,
    }
}
