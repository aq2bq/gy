//! The shared generator and replay for the invariant property tests (n-99c6).
//! A plan is one random sequence of writes; replay records what every step saw
//! and did. P1b, P2 and P3 read this module with `mod prop;`.
use gy_ledger::{Actor, FormatVersion, MemoryStore, Node, NodeId, Repository, Store};
use proptest::prelude::*;
use std::collections::BTreeSet;

pub mod rules;
mod step;
use step::{creates, run};

pub const CASES: u32 = 96;
pub const MAX_OPS: usize = 24;

/// A rule that can refuse a write among E1–E4. E2 is a state invariant: no
/// write is refused for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    E1,
    E3,
    E4,
}

/// A write the generator can make. `a` and `b` pick an operand by position in
/// the order the nodes were created (8 or more means an id that does not
/// exist, E4); `rel` indexes `rules::ALLOWED`; `mark` 0 is no mark; `flag`
/// selects a variant.
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

/// The state the rules need: criteria, needs, a filed requirement, a question
/// and a decision, so an operation has targets and an edge to repeat.
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
        op(Kind::QuestionAdd, 0, 0),
        op(Kind::NeedAdd, 0, 1),
        op(Kind::NeedAdd, 1, 0),
        op(Kind::ReqAdd, 0, 1),
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

/// The rule a write is in the domain of, from its kind alone: enough to see
/// that an item was exercised (a count of zero fails the item).
pub fn hits(op: &Op) -> BTreeSet<Rule> {
    let mut set = BTreeSet::new();
    match op.kind {
        Kind::Stray => {
            set.insert(Rule::E1);
        }
        Kind::Repeat => {
            set.insert(Rule::E3);
        }
        _ => {}
    }
    if op.a >= 8 || op.b >= 8 {
        set.insert(Rule::E4);
    }
    set
}
