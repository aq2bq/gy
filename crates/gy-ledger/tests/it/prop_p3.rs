//! The P3 properties over random operation sequences (n-7a58): the derived
//! judgements Nd3–Nd5, Q3, C3, C4, D2 and V1–V5, checked after every step
//! from the empty record. Each item compares the library's answer (`show`,
//! `list`, `next`, `now`, `handover`) with the spec's own function in
//! `derived.rs`, so a semantic change in the source shows up here.
use crate::derived;
use crate::prop;

use derived as d;
use gy_ledger::{
    Filter, Listing, MemoryStore, Node, NodeData, NodeKind, Repository, Shown, advice_for,
    bearer_count, handover, list, next, now, show,
};
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use std::cell::Cell;

/// Replay the plan and hand every step's state (its nodes and the repository)
/// to `visit`, from the empty record.
fn run_states(visit: impl FnMut(&[Node], &Repository<MemoryStore>) -> Result<(), TestCaseError>) {
    let visit = std::cell::RefCell::new(visit);
    TestRunner::new(Config::with_cases(prop::CASES))
        .run(&prop::plan(), |plan| {
            prop::replay_visit(&plan, |_step, repo| {
                let all = repo.all().unwrap();
                (visit.borrow_mut())(&all, repo)
            })?;
            Ok(())
        })
        .unwrap();
}

/// Every node as `show` projects it, in the order of `all`.
fn shown_all(all: &[Node], repo: &Repository<MemoryStore>) -> Vec<Shown> {
    let ids: Vec<String> = all.iter().map(|node| node.id().to_string()).collect();
    show(repo, &ids, true).unwrap()
}

fn needs(all: &[Node]) -> Vec<Node> {
    all.iter()
        .filter(|node| node.kind() == NodeKind::Need)
        .cloned()
        .collect()
}

/// Nd3: the state `show` reports for every need is the derived one, and it is
/// the same in `list`.
#[test]
fn nd3_need_state() {
    let derived = Cell::new(0usize);
    run_states(|all, repo| {
        let shown = shown_all(all, repo);
        let Listing::Nodes(rows) = list(repo, &Filter::default()).unwrap() else {
            return Ok(());
        };
        for (need, shown) in all.iter().zip(&shown) {
            if need.kind() != NodeKind::Need {
                continue;
            }
            derived.set(derived.get() + 1);
            prop_assert_eq!(shown.need, Some(d::need_state(need, all)));
            let row = rows.iter().find(|row| row.id == need.id().to_string());
            prop_assert_eq!(row.and_then(|row| row.status.as_deref()), shown.need);
        }
        Ok(())
    });
    assert!(derived.get() > 0, "ND3: no need was derived");
}

/// Nd4: no need the source calls done has an unmet targeted criterion.
#[test]
fn nd4_done_need_has_no_unmet_criterion() {
    let done = Cell::new(0usize);
    run_states(|all, repo| {
        for (need, shown) in all.iter().zip(&shown_all(all, repo)) {
            if need.kind() == NodeKind::Need && shown.need == Some("done") {
                done.set(done.get() + 1);
                prop_assert!(
                    d::done_has_no_unmet(need, all),
                    "done need {} still has an unmet criterion",
                    need.id()
                );
            }
        }
        Ok(())
    });
    assert!(done.get() > 0, "ND4: no need was done");
}

/// Nd5: `next` lists exactly the ready needs.
#[test]
fn nd5_ready() {
    let ready = Cell::new(0usize);
    run_states(|all, repo| {
        let mut got: Vec<String> = next(repo, None)
            .unwrap()
            .iter()
            .map(|r| r.id.clone())
            .collect();
        let mut want = d::ready_ordered(all);
        got.sort();
        want.sort();
        ready.set(ready.get() + got.len());
        prop_assert_eq!(got, want);
        Ok(())
    });
    assert!(ready.get() > 0, "ND5: no need was ready");
}

/// Q3: a question is open iff it has no closure, as `list` and `handover`
/// report it.
#[test]
fn q3_question_open() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        let Listing::Nodes(rows) = list(repo, &Filter::default()).unwrap() else {
            return Ok(());
        };
        let mut open = 0;
        for question in all.iter().filter(|node| node.kind() == NodeKind::Question) {
            let row = rows.iter().find(|row| row.id == question.id().to_string());
            let want = if d::question_open(question) {
                open += 1;
                "open"
            } else {
                "closed"
            };
            checked.set(checked.get() + 1);
            prop_assert_eq!(row.and_then(|row| row.status.as_deref()), Some(want));
        }
        prop_assert_eq!(handover(repo, None).unwrap().open_questions, open);
        Ok(())
    });
    assert!(checked.get() > 0, "Q3: no question was judged");
}

/// C3: the rule's coverage and the advice's coverage are one function: the
/// advice's missing line and the advice's next step both name the same
/// `covered` the reference computes.
#[test]
fn c3_covered_is_one_function() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        for criterion in all.iter().filter(|node| node.kind() == NodeKind::Criterion) {
            if d::criterion_satisfied(criterion) {
                continue;
            }
            checked.set(checked.get() + 1);
            let covered = d::covered(all, criterion.id());
            let (missing, steps) = advice_for(repo, criterion).unwrap();
            let missing_says = !missing
                .iter()
                .any(|line| line == "an approved requirement (targets)");
            let step_says = steps
                .first()
                .is_some_and(|step| step.starts_with("criterion satisfy"));
            prop_assert_eq!(missing_says, covered);
            prop_assert_eq!(step_says, covered);
            prop_assert_eq!(missing_says, step_says);
        }
        Ok(())
    });
    assert!(checked.get() > 0, "C3: no criterion was judged");
}

/// C4: `bearer_count` counts needs, and an orphaned criterion (unmet, borne,
/// every bearer closed by hand) is the count `handover` reports and the lines a
/// closed need reports.
#[test]
fn c4_bearer_count_and_orphaned() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        let shown = shown_all(all, repo);
        let needs = needs(all);
        for criterion in all.iter().filter(|node| node.kind() == NodeKind::Criterion) {
            checked.set(checked.get() + 1);
            prop_assert_eq!(
                bearer_count(&needs, criterion.id()),
                d::bearers(all, criterion.id()).len()
            );
        }
        let want = all
            .iter()
            .filter(|node| node.kind() == NodeKind::Criterion && d::orphaned(node, all))
            .count();
        let got = handover(repo, None)
            .unwrap()
            .warnings
            .iter()
            .find(|warning| warning.label == "criteria unmet with every need closed")
            .map_or(0, |warning| warning.count);
        prop_assert_eq!(got, want);
        for (need, shown) in all.iter().zip(&shown) {
            if !d::closed_by_hand(need) {
                continue;
            }
            let want: Vec<String> = d::targets(need)
                .into_iter()
                .filter(|id| d::find(all, id).is_some_and(|node| d::orphaned(node, all)))
                .map(|id| format!("unmet criterion {id}"))
                .collect();
            prop_assert_eq!(&shown.missing, &want);
        }
        Ok(())
    });
    assert!(checked.get() > 0, "C4: no criterion was judged");
}

/// D2: `show` returns the stored body with the narrowed passages marked, beside
/// the retractions the graph carries; the mark that resolves is the one the
/// older decision's body holds.
#[test]
fn d2_retraction_derived() {
    let checked = Cell::new(0usize);
    let narrowed = Cell::new(0usize);
    run_states(|all, repo| {
        for (node, shown) in all.iter().zip(&shown_all(all, repo)) {
            checked.set(checked.get() + 1);
            let want = d::retraction(all, &node.id().to_string());
            prop_assert_eq!(&shown.cancellation, &want);
            let body_marked = want.as_ref().and_then(|r| d::retracted(node.body(), r));
            prop_assert_eq!(&shown.body_marked, &body_marked);
            let scope_marked = match (node.data(), want.as_ref()) {
                (NodeData::Decision(data), Some(retraction)) => {
                    d::retracted(data.scope.text(), retraction)
                }
                _ => None,
            };
            prop_assert_eq!(&shown.decision_scope_marked, &scope_marked);
            if want.as_ref().is_some_and(|r| !r.narrowed.is_empty()) {
                narrowed.set(narrowed.get() + 1);
            }
        }
        Ok(())
    });
    assert!(checked.get() > 0, "D2: no node was judged");
    assert!(narrowed.get() > 0, "D2: no narrowed passage was exercised");
}

/// V1: the views judge the same derived values: `show` and `list` agree on a
/// need's state, `next`, `now` and `handover` agree on the ready needs, and
/// `handover`'s open-question count is the questions' own closure.
#[test]
fn v1_views_share_one_function() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        let shown = shown_all(all, repo);
        let Listing::Nodes(rows) = list(repo, &Filter::default()).unwrap() else {
            return Ok(());
        };
        for (node, shown) in all.iter().zip(&shown) {
            if node.kind() != NodeKind::Need {
                continue;
            }
            checked.set(checked.get() + 1);
            let row = rows.iter().find(|row| row.id == node.id().to_string());
            prop_assert_eq!(row.and_then(|row| row.status.as_deref()), shown.need);
        }
        let ordered: Vec<String> = next(repo, None)
            .unwrap()
            .iter()
            .map(|r| r.id.clone())
            .collect();
        let ready: Vec<String> = now(repo, None)
            .unwrap()
            .ready
            .iter()
            .map(|row| row.row.id.clone())
            .collect();
        let session = handover(repo, None).unwrap();
        prop_assert_eq!(&ordered, &ready);
        prop_assert_eq!(ordered.len(), session.ready_needs);
        let open = all
            .iter()
            .filter(|node| node.kind() == NodeKind::Question && d::question_open(node))
            .count();
        prop_assert_eq!(session.open_questions, open);
        Ok(())
    });
    assert!(checked.get() > 0, "V1: no view was read");
}

/// V2: `missing` is the spec's gaps by kind and state, in `show` and in the
/// advice a write returns.
#[test]
fn v2_missing_by_kind_and_state() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        for (node, shown) in all.iter().zip(&shown_all(all, repo)) {
            checked.set(checked.get() + 1);
            let want = d::missing(node, all);
            prop_assert_eq!(&shown.missing, &want);
            let (advice_missing, _) = advice_for(repo, node).unwrap();
            prop_assert_eq!(&advice_missing, &want);
        }
        Ok(())
    });
    assert!(checked.get() > 0, "V2: no node was judged");
}

/// V3: the step toward an unmet criterion is the spec's: satisfy when covered,
/// approve a filed requirement when one targets it, add a requirement bearing
/// the need otherwise; the body edit leads when the body is empty.
#[test]
fn v3_criterion_step() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        for node in all
            .iter()
            .filter(|node| matches!(node.kind(), NodeKind::Need | NodeKind::Criterion))
        {
            checked.set(checked.get() + 1);
            let (_, steps) = advice_for(repo, node).unwrap();
            prop_assert_eq!(&steps, &d::next(node, all));
        }
        Ok(())
    });
    assert!(checked.get() > 0, "V3: no criterion or need was judged");
}

/// V4: `next` lists the ready needs ordered by created then ID, and `now`
/// orders them the same way.
#[test]
fn v4_next_order() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        let want = d::ready_ordered(all);
        let got: Vec<String> = next(repo, None)
            .unwrap()
            .iter()
            .map(|r| r.id.clone())
            .collect();
        checked.set(checked.get() + got.len());
        prop_assert_eq!(&got, &want);
        let ready: Vec<String> = now(repo, None)
            .unwrap()
            .ready
            .iter()
            .map(|row| row.row.id.clone())
            .collect();
        prop_assert_eq!(&ready, &want);
        Ok(())
    });
    assert!(checked.get() > 0, "V4: no ready need was ordered");
}

/// V5: handover's warnings are the spec's counts, in the spec's order.
#[test]
fn v5_handover_warnings() {
    let checked = Cell::new(0usize);
    run_states(|all, repo| {
        checked.set(checked.get() + 1);
        prop_assert_eq!(&handover(repo, None).unwrap().warnings, &d::warnings(all));
        Ok(())
    });
    assert!(checked.get() > 0, "V5: no state was judged");
}
