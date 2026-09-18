//! `list` takes the words as the screen shows them, and `--since` a date
//! (n-a56f, r-2aa4, n-b6b6): case does not matter, a date means a local day's
//! start, and an unknown word names the ones it takes.
mod common;

use common::{fixture, id_of, stderr, stdout};

fn rows(fx: &common::Fixture, args: &[&str]) -> String {
    let out = fx.run(args);
    assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    stdout(&out)
}

#[test]
fn since_uses_the_local_day_start() {
    let fx = fixture();
    let ledger = fx.ledger();
    gy_ledger::format::write(&ledger, gy_ledger::FormatVersion::CURRENT).unwrap();
    let node = gy_ledger::Node::need(
        gy_ledger::NodeId::from_hash(gy_ledger::NodeKind::Need, "0001").unwrap(),
        "a",
        "2026-09-15T23:00:00Z",
        "a need",
    )
    .unwrap();
    // One write at 2026-09-15T23:30:00Z: still the 15th in UTC, already the
    // 16th at UTC+2.
    let event = gy_ledger::log::Event {
        seq: 1,
        at: 1_789_515_000,
        actor: "piko".into(),
        why: "need add".into(),
        source: "test".into(),
        changes: vec![gy_ledger::log::Change::Created {
            node: "n-0001".into(),
            value: serde_json::to_value(&node).unwrap(),
        }],
    };
    gy_ledger::log::append(&ledger, &event).unwrap();

    let run = |tz: &str| {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_gy"))
            .current_dir(&fx.root)
            .env("XDG_DATA_HOME", &fx.data)
            .env("GY_ACTOR", "piko")
            .env("TZ", tz)
            .args(["list", "--since", "2026-09-16", "--json"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{tz}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    assert_eq!(run("UTC"), "[]\n", "the write is before the 16th in UTC");
    assert!(
        run("Etc/GMT-2").contains("n-0001"),
        "the write is on the 16th at UTC+2"
    );
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
