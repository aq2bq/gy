//! n-fe59: two writers on one ledger. The retry keeps every write, and the
//! lines that needed one carry the count.
mod common;

use common::{Fixture, fixture};
use std::process::{Command, Output};
use std::sync::{Arc, Barrier};
use std::thread;

fn write(root: &std::path::Path, data: &std::path::Path, title: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gy"))
        .current_dir(root)
        .env("XDG_DATA_HOME", data)
        .env("GY_ACTOR", "piko")
        .args(["criterion", "add", title])
        .output()
        .unwrap()
}

/// Two writers in lockstep, so their writes meet on the ledger rather than
/// running one after the other.
fn two_writers(fx: &Fixture) -> (usize, usize) {
    let barrier = Arc::new(Barrier::new(2));
    let spawn = |prefix: &'static str| {
        let root = fx.root.clone();
        let data = fx.data.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let mut ok = 0;
            for index in 0..20 {
                barrier.wait();
                if write(&root, &data, &format!("{prefix}{index}"))
                    .status
                    .success()
                {
                    ok += 1;
                }
            }
            ok
        })
    };
    let first = spawn("a");
    let second = spawn("b");
    (first.join().unwrap(), second.join().unwrap())
}

#[test]
fn two_writers_do_not_lose_writes() {
    let fx = fixture();
    let (first, second) = two_writers(&fx);
    assert_eq!(first + second, 40);

    let text = std::fs::read_to_string(fx.ledger().join("events.jsonl")).unwrap();
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 40, "a write did not reach the log");

    let retries: u64 = lines
        .iter()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .map(|event| event["retries"].as_u64().unwrap_or(0))
        .sum();
    assert!(
        retries >= 1,
        "no writer ever collided; the test proves nothing"
    );
}
