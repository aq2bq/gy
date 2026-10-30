//! The measurement ac-66f1 asks for, on the master's copy of the kokopelli
//! ledger (seq 4853): the three cuts — the largest scope, the word that hits
//! the most nodes, the busiest week — each answered within a second, and no
//! coordinate anywhere (d-6d90 removed the server-side layout).
//!
//! The copy is the master's own work and never in git. The test reads the
//! directory named by `GY_KOKOPELLI_LEDGER` (a ledger directory holding
//! `format`, `events.jsonl`, and `snapshot.json`), says one line and passes
//! when it is unset, and prints what it measured when it runs.
use gy_ledger::{FileStore, Node, Repository};
use gy_serve::api::route;
use gy_serve::http::Request;
use gy_serve::server::{self, Opened, Opener};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The most a cut may take before ac-66f1 says it failed.
const LIMIT: Duration = Duration::from_secs(1);

#[test]
fn the_three_questions_answer_within_a_second() {
    let Some(dir) = std::env::var_os("GY_KOKOPELLI_LEDGER").map(PathBuf::from) else {
        eprintln!("GY_KOKOPELLI_LEDGER is unset; the kokopelli measurement is skipped");
        return;
    };
    let open: Opener =
        Box::new(move |_| FileStore::open(&dir).map(|store| Opened::Now(Repository::new(store))));
    let Some(questions) = questions(&open) else {
        panic!("the ledger answers none of the three questions");
    };
    let cuts = [
        (
            "largest scope",
            format!("scope={}", encode(&questions.scope)),
        ),
        ("busiest word", format!("q={}", encode(&questions.word))),
        (
            "busiest week",
            format!(
                "from={}&to={}",
                questions.week.format("%Y-%m-%d"),
                (questions.week + chrono::Days::new(6)).format("%Y-%m-%d")
            ),
        ),
    ];
    for (what, query) in cuts {
        let request = Request::new("GET", "/api/cross", Some(&query), &[]);
        let started = Instant::now();
        let response = server::answer(&open, &request, "gy");
        let elapsed = started.elapsed();
        assert_eq!(response.status, 200, "{what}: {query}");
        let json: serde_json::Value = serde_json::from_slice(&response.body).unwrap();
        // No coordinates: the answer is a cut, and the screen places it. A
        // non-finite number cannot even be written by serde_json, so a parsed
        // body is finite by construction; the nodes carry no x/y at all.
        if let Some(nodes) = json["nodes"].as_array() {
            for node in nodes {
                assert!(node.get("x").is_none() && node.get("y").is_none(), "{what}");
            }
        }
        eprintln!(
            "{what}: {} in {:.3}s ({} nodes, capped {})",
            query,
            elapsed.as_secs_f64(),
            json["nodes"].as_array().map_or(0, Vec::len),
            json["capped"]
        );
        assert!(
            elapsed < LIMIT,
            "{what}: {query} took {:.3}s",
            elapsed.as_secs_f64()
        );
    }
}

/// The questions the measurement asks, found from the ledger itself.
struct Questions {
    scope: String,
    word: String,
    week: chrono::NaiveDate,
}

/// The largest scope, the word that hits the most nodes, and the week with the
/// most moves — read from one open ledger.
fn questions(open: &Opener) -> Option<Questions> {
    let Opened::Now(repo) = open(None).ok()? else {
        return None;
    };
    let nodes = repo.all().ok()?;
    let scope = largest_scope(&nodes)?;
    let word = busiest_word(&nodes);
    let grid = route(&repo, &Request::new("GET", "/api/cross", None, &[]), "gy");
    let json: serde_json::Value = serde_json::from_slice(&grid.body).ok()?;
    let week = busiest_week(&json)?;
    Some(Questions { scope, word, week })
}

/// The scope holding the most nodes, ties by name.
fn largest_scope(nodes: &[Node]) -> Option<String> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for node in nodes {
        *counts.entry(node.scope()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(scope, _)| scope.to_string())
}

/// The word a node's title or body carries that the most nodes carry too. A
/// token is a run of letters or digits; the longest runs a Japanese sentence
/// makes are left out, so this finds the names and terms the record shares.
fn busiest_word(nodes: &[Node]) -> String {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for node in nodes {
        let text = format!("{} {}", node.title(), node.body());
        let mut here: HashSet<String> = HashSet::new();
        for token in text.split(|c: char| !c.is_alphanumeric()) {
            let token = token.to_lowercase();
            let length = token.chars().count();
            // A year or a bare number is not a word a person searches for.
            if !(3..=24).contains(&length) || !token.chars().any(char::is_alphabetic) {
                continue;
            }
            if here.insert(token.clone()) {
                *counts.entry(token).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(word, _)| word)
        .unwrap_or_default()
}

/// The Monday-start week with the most moves, from the unfiltered grid.
fn busiest_week(json: &serde_json::Value) -> Option<chrono::NaiveDate> {
    let mut totals: HashMap<String, u64> = HashMap::new();
    for week in json["weeks"].as_array()? {
        let from = week["from"].as_str()?.to_string();
        let moves = week["moves"].as_u64()?;
        *totals.entry(from).or_default() += moves;
    }
    let (from, _) = totals
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))?;
    chrono::NaiveDate::parse_from_str(&from, "%Y-%m-%d").ok()
}

/// A query value as a browser writes it.
fn encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
