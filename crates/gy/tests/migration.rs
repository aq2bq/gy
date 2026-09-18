//! A format migration is reported once (n-f8a2): the store says it happened,
//! and the CLI says it on stderr. The store check lives here because
//! `crates/gy-ledger/tests/format.rs` is at its 300-line limit.
mod common;

use common::{fixture, stderr, stdout};
use gy_ledger::{FileStore, FormatVersion, format};

#[test]
fn an_open_that_migrates_reports_it_once() {
    let temp = tempfile::tempdir().unwrap();
    format::write(temp.path(), FormatVersion(2)).unwrap();

    let first = FileStore::open_with(temp.path(), |_| Some("piko".into())).unwrap();
    assert_eq!(first.migrated(), Some((2, 3)));
    assert_eq!(format::read(temp.path()).unwrap(), FormatVersion::CURRENT);

    let second = FileStore::open_with(temp.path(), |_| Some("piko".into())).unwrap();
    assert_eq!(second.migrated(), None);
}

#[test]
fn the_cli_announces_a_migration_on_stderr_once() {
    let fx = fixture();
    fx.seed(&[common::need("0001", "a need")]);
    // The ledger is marked as format 2; the next open migrates it to 3.
    std::fs::write(fx.ledger().join("format"), "2\n").unwrap();

    let first = fx.run(&["show", "n-0001"]);
    assert!(first.status.success(), "{}", stderr(&first));
    assert!(
        stderr(&first).contains("migrated this ledger from format 2 to 3"),
        "{}",
        stderr(&first)
    );
    assert!(stderr(&first).contains("CHANGELOG"), "{}", stderr(&first));

    // Standard output is unchanged, and the notice is only on the first open.
    let second = fx.run(&["show", "n-0001"]);
    assert_eq!(stdout(&second), stdout(&first));
    assert!(stderr(&second).is_empty(), "{}", stderr(&second));
}
