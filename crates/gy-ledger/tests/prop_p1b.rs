//! The E5–E8, R5, C1, C2 and T5 properties over the same random operation
//! sequences as P1a (n-a006). A rule item checks every accepted step's state
//! change and counts the refused steps that met the rule; T5 is a state item
//! and reads a column that includes undo. A zero count fails, so an item that
//! was never exercised is visible.
mod prop;
#[path = "prop/rules.rs"]
mod rules;
#[path = "prop/rules_e.rs"]
mod rules_e;
use prop::{Kind, Rule, State, Step};
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use std::cell::Cell;

/// Every state invariant E1–E8, for T5's "the gate runs on every path".
fn state_broken(before: &State, after: &State) -> bool {
    rules::broken_e1(before, after)
        || rules::broken_e2(before, after)
        || rules::broken_e3(before, after)
        || rules::broken_e4(before, after)
        || rules_e::broken_e5(before, after)
        || rules_e::broken_e6(before, after)
        || rules_e::broken_e7(before, after)
        || rules_e::broken_e8(before, after)
}

/// Replay the plan many times, handing each step to `visit`.
fn run_all(visit: impl Fn(&Step) -> Result<(), TestCaseError>) {
    TestRunner::new(Config::with_cases(prop::CASES))
        .run(&prop::plan(), |plan| {
            prop::replay_visit(&plan, |step, _| visit(step))
        })
        .unwrap();
}

/// Check accepted steps against `check`, refused steps against T1, and count
/// refused steps that met `rule`. Returns (rule-hit refusals, accepted steps).
fn check(check: fn(&State, &State) -> bool, rule: Option<Rule>) -> (usize, usize) {
    let hits = Cell::new(0usize);
    let accepted = Cell::new(0usize);
    run_all(|step| {
        if step.accepted {
            accepted.set(accepted.get() + 1);
            prop_assert!(!check(&step.before, &step.after));
        } else {
            prop_assert_eq!(&step.before, &step.after);
            if let Some(rule) = rule {
                if step.hits.contains(&rule) {
                    hits.set(hits.get() + 1);
                }
            }
        }
        Ok(())
    });
    (hits.get(), accepted.get())
}

fn rule_test(name: &str, predicate: fn(&State, &State) -> bool, rule: Rule) {
    let (hits, accepted) = check(predicate, Some(rule));
    eprintln!("{name}: refused-hits={hits} accepted={accepted}");
    assert!(hits > 0, "{name}: no refused step met the rule");
    assert!(accepted > 0, "{name}: no accepted step");
}

macro_rules! rule_items {
    ($($name:ident = $rule:ident <- $pred:path;)*) => {$(
        #[test]
        fn $name() { rule_test(stringify!($rule), $pred, Rule::$rule); }
    )*};
}

rule_items! {
    e5_marks_resolve = E5 <- rules_e::broken_e5;
    e6_closes_from_decision = E6 <- rules_e::broken_e6;
    e7_graphs_acyclic = E7 <- rules_e::broken_e7;
    e8_closed_need_filed_as = E8 <- rules_e::broken_e8;
    r5_approved_frozen = R5 <- rules_e::broken_r5;
    c1_covered_satisfy = C1 <- rules_e::broken_c1;
    c2_evidence_once = C2 <- rules_e::broken_c2;
}

/// T5: the gate runs on every path, undo included; every accepted step leaves
/// all of E1–E8 standing.
#[test]
fn t5_gate_every_path() {
    let accepted = Cell::new(0usize);
    let undos = Cell::new(0usize);
    run_all(|step| {
        if step.accepted {
            accepted.set(accepted.get() + 1);
            if step.kind == Kind::Undo {
                undos.set(undos.get() + 1);
            }
            prop_assert!(!state_broken(&step.before, &step.after));
        } else {
            prop_assert_eq!(&step.before, &step.after);
        }
        Ok(())
    });
    assert!(accepted.get() > 0, "T5: no accepted step");
    assert!(undos.get() > 0, "T5: no accepted undo");
}
