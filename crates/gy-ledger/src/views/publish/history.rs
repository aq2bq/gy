//! The write history in range for one scope: one row per write unit.
use super::reference;
use crate::model::Node;
use crate::ops::repository::{Repository, Result, Store};
use std::collections::BTreeMap;
use std::fmt::Write;

const HEADER: &str = "| seq | when | writer | node | what | why | source |\n\
                      |---|---|---|---|---|---|---|\n";

pub(super) fn history<S: Store>(
    repository: &Repository<S>,
    all: &[Node],
    scope: &str,
    since: Option<u64>,
) -> Result<String> {
    let known: BTreeMap<_, _> = all
        .iter()
        .map(|node| (node.id().to_string(), node))
        .collect();
    let mut out = String::from(HEADER);
    let mut count = 0;
    for entry in repository.store().history() {
        if since.is_some_and(|since| entry.seq <= since) {
            continue;
        }
        let label = match known.get(&entry.node) {
            Some(node) if node.scope() == scope => reference(node),
            Some(_) => continue,
            None if entry.node.is_empty() => String::new(),
            None => continue,
        };
        count += 1;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            entry.seq,
            time(entry.at),
            cell(entry.actor.name()),
            cell(&label),
            cell(&entry.what),
            cell(&entry.why),
            cell(&entry.source)
        );
    }
    if count == 0 {
        out.push_str("(no history)\n");
    }
    Ok(out)
}

fn time(at: u64) -> String {
    chrono::DateTime::from_timestamp(at as i64, 0)
        .map(|time| time.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| at.to_string())
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|")
}
