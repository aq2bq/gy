//! The change section: the write units after a sequence, in time order, with
//! decision creation and question closure standing out (D-85 question 5).
use super::super::list::{Filter, Listing, list};
use super::label;
use crate::ops::repository::{Repository, Result, Store};
use std::fmt::Write;

pub(super) fn changes<S: Store>(repository: &Repository<S>, since: u64) -> Result<String> {
    let filter = Filter {
        since: Some(since),
        ..Filter::default()
    };
    let Listing::History(rows) = list(repository, &filter)? else {
        return Ok("変更は無し。\n".to_string());
    };
    if rows.is_empty() {
        return Ok("変更は無し。\n".to_string());
    }
    let mut out = String::new();
    for row in rows {
        let node = repository
            .resolve(&row.node)
            .ok()
            .and_then(|id| repository.get(&id).ok().flatten());
        let (label, title) = match &node {
            Some(node) => (label(node), node.title().to_string()),
            None => (row.node.clone(), "（削除）".to_string()),
        };
        let _ = writeln!(
            out,
            "- {} {}: {}{} {} — {} / {}",
            time(row.at),
            row.actor,
            emphasis(&row.why),
            label,
            title,
            row.why,
            row.source
        );
    }
    Ok(out)
}

/// Decision creation and question closure are the events a reader looks for.
fn emphasis(why: &str) -> &'static str {
    if why.starts_with("decide") {
        "【決定の作成】"
    } else if why.starts_with("question close") {
        "【論点の閉じ】"
    } else {
        ""
    }
}

fn time(at: u64) -> String {
    chrono::DateTime::from_timestamp(at as i64, 0)
        .map(|time| time.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| at.to_string())
}
