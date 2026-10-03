//! The E1–E4 properties over random operation sequences (n-99c6, P1a). A rule
//! item checks every accepted step's state change and counts the refused steps
//! that met the rule; E2 is a state item. A zero count fails, so an item that
//! was never exercised is visible.
mod prop;
use prop::{Rule, State, Step, rules};
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use std::cell::Cell;

/// Replay the plan many times, handing each step to `visit`.
fn run_all(visit: impl Fn(&Step) -> Result<(), TestCaseError>) {
    TestRunner::new(Config::with_cases(prop::CASES))
        .run(&prop::plan(), |plan| {
            for step in &prop::replay(&plan).steps {
                visit(step)?;
            }
            Ok(())
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
    e1_relation_pairs = E1 <- rules::broken_e1;
    e3_duplicate_edge = E3 <- rules::broken_e3;
    e4_target_exists = E4 <- rules::broken_e4;
}

/// E2 is a state item only: no write is refused by it.
#[test]
fn e2_edges_stored_forward() {
    let (_, accepted) = check(rules::broken_e2, None);
    assert!(accepted > 0, "E2: no accepted step");
}
