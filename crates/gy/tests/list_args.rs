//! `list` takes the words as the screen shows them, and `--since` a date
//! (n-a56f, r-2aa4): case does not matter, a date means a UTC day's start, and
//! an unknown word names the ones it takes.
mod common;

use common::{fixture, id_of, stderr, stdout};

fn rows(fx: &common::Fixture, args: &[&str]) -> String {
    let out = fx.run(args);
    assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    stdout(&out)
}

#[test]
fn the_words_ignore_case_and_since_takes_a_date() {
    let fx = fixture();
    let create = |args: &[&str]| {
        let out = fx.run(args);
        assert!(out.status.success(), "{args:?}: {}", stderr(&out));
        id_of(&out)
    };
    let criterion = create(&["criterion", "add", "measures one"]);
    let need = create(&["need", "add", "the need", "--targets", &criterion]);
    create(&[
        "question",
        "add",
        "a question",
        "--decider",
        "master",
        "--options",
        "one",
        "--options",
        "two",
    ]);
    create(&["decide", "a decision", "--scope-note", "a call"]);
    create(&["req", "add", "a requirement", "--need", &need]);

    for kind in ["Need", "Question", "Decision", "Requirement", "Criterion"] {
        let shown = rows(&fx, &["list", "--type", kind, "--json"]);
        assert_eq!(
            shown,
            rows(&fx, &["list", "--type", &kind.to_lowercase(), "--json"]),
            "{kind} lowercase"
        );
        assert_eq!(
            shown,
            rows(&fx, &["list", "--type", &kind.to_uppercase(), "--json"]),
            "{kind} uppercase"
        );
    }
    assert_eq!(
        rows(&fx, &["list", "--status", "Open", "--json"]),
        rows(&fx, &["list", "--status", "open", "--json"])
    );

    // A date and a sequence that mean the same thing give the same rows: a day
    // long past is every write, a day far ahead is none (the day's arithmetic
    // holds outside the ledger's range too).
    assert_eq!(
        rows(&fx, &["list", "--since", "2000-01-01", "--json"]),
        rows(&fx, &["list", "--since", "0", "--json"])
    );
    assert_eq!(
        rows(&fx, &["list", "--since", "2099-01-01", "--json"]),
        "[]\n"
    );
}

#[test]
fn an_unknown_word_names_the_ones_it_takes() {
    let fx = fixture();
    let out = fx.run(&["criterion", "add", "measures one"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let out = fx.run(&["list", "--type", "Whatever"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    for word in ["Need", "Question", "Decision", "Requirement", "Criterion"] {
        assert!(message.contains(word), "{message}");
    }

    let out = fx.run(&["list", "--status", "Whatever"]);
    assert_eq!(out.status.code(), Some(2));
    let message = stderr(&out);
    for word in ["open", "closed", "satisfied", "cancelled"] {
        assert!(message.contains(word), "{message}");
    }
}
