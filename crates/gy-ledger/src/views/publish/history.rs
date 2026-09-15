//! The write history in range: one row per write unit, in time order, with the
//! node's title beside its ID (d-edb0).
use super::reference;
use crate::ops::repository::{Repository, Result, Store};
use std::collections::BTreeMap;
use std::fmt::Write;

pub(super) fn history<S: Store>(repository: &Repository<S>, since: Option<u64>) -> Result<String> {
    let known: BTreeMap<String, String> = repository
        .all()?
        .into_iter()
        .map(|node| (node.id().to_string(), reference(&node)))
        .collect();
    let mut out = String::from(
        "| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |\n\
         |---|---|---|---|---|---|---|\n",
    );
    let mut count = 0;
    for entry in repository.store().history() {
        if since.is_some_and(|since| entry.seq <= since) {
            continue;
        }
        count += 1;
        let node = known
            .get(&entry.node)
            .cloned()
            .unwrap_or_else(|| format!("{}（削除）", entry.node));
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            entry.seq,
            time(entry.at),
            cell(entry.actor.name()),
            cell(&node),
            cell(&entry.what),
            cell(&entry.why),
            cell(&entry.source)
        );
    }
    if count == 0 {
        out.push_str("（履歴は無し）\n");
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
