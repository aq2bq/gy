//! The serve log's failure throttle (n-08ae, ac-11c9): the first failure with
//! its advice, then every tenth round, and one recovery line.
use gy_ledger::Rounds;

#[test]
fn the_first_failure_prints_two_lines() {
    let mut rounds = Rounds::new();
    let lines = rounds.failed(
        "the sync gave up after 10 seconds while fetching the remote",
        Some("nothing to do now"),
        "01:00:00",
    );
    assert_eq!(
        lines,
        [
            "01:00:00 sync failed: the sync gave up after 10 seconds while fetching the remote",
            "  → nothing to do now",
        ]
    );
}

#[test]
fn a_failure_without_advice_prints_one_line() {
    let mut rounds = Rounds::new();
    assert_eq!(
        rounds.failed("boom", None, "00:00:00"),
        ["00:00:00 sync failed: boom"]
    );
}

#[test]
fn the_same_failure_is_quiet_until_the_eleventh_round() {
    let mut rounds = Rounds::new();
    assert_eq!(rounds.failed("boom", Some("fix it"), "01:00:00").len(), 2);
    for round in 2..=10 {
        assert!(
            rounds.failed("boom", Some("fix it"), "01:00:10").is_empty(),
            "round {round} is quiet"
        );
    }
    assert_eq!(
        rounds.failed("boom", Some("fix it"), "01:01:40"),
        ["01:01:40 still failing since 01:00:00 (boom)"]
    );
}

#[test]
fn a_changed_failure_prints_again() {
    let mut rounds = Rounds::new();
    rounds.failed("one", Some("fix one"), "00:00:00");
    let lines = rounds.failed("two", Some("fix two"), "00:00:10");
    assert_eq!(lines, ["00:00:10 sync failed: two", "  → fix two"]);
}

#[test]
fn recovery_prints_once_after_the_failures() {
    let mut rounds = Rounds::new();
    rounds.failed("boom", Some("fix"), "00:00:00");
    rounds.failed("boom", Some("fix"), "00:00:10");
    assert_eq!(
        rounds.recovered("00:00:20"),
        ["00:00:20 sync: recovered after 2 failed rounds"]
    );
    assert!(rounds.recovered("00:00:30").is_empty());
}
