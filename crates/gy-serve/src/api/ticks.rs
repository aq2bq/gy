//! GET /api/ticks: one tick per write of the whole log, for the time scrubber
//! (n-10e1). The log is the ledger, so this is only a lighter read of it.
use crate::http::{Request, Response};
use gy_ledger::{Repository, Store};
use serde::Serialize;

/// One write: its sequence, when, who, and the kind of what it did.
#[derive(Serialize)]
struct Tick {
    seq: u64,
    at: u64,
    actor: String,
    kind: &'static str,
}

#[derive(Serialize)]
struct Ticks {
    max: u64,
    ticks: Vec<Tick>,
}

pub fn ticks<S: Store>(repo: &Repository<S>, _req: &Request) -> Response {
    let mut ticks: Vec<Tick> = Vec::new();
    for entry in repo.store().history() {
        // One tick per write, not per changed node (a write may touch several).
        if ticks.last().is_some_and(|tick| tick.seq == entry.seq) {
            continue;
        }
        ticks.push(Tick {
            seq: entry.seq,
            at: entry.at,
            actor: entry.actor.name().to_string(),
            kind: kind(&entry.why),
        });
    }
    Response::json(&Ticks {
        max: ticks.last().map_or(0, |tick| tick.seq),
        ticks,
    })
}

/// The kind of a write, from the first word of the why it carries.
fn kind(why: &str) -> &'static str {
    match why.split_whitespace().next().unwrap_or_default() {
        "decide" => "decide",
        "need" => "need",
        "question" => "question",
        "criterion" => "criterion",
        "link" => "link",
        "edit" => "edit",
        "req" => "req",
        "scope" => "scope",
        "undo" => "undo",
        _ => "other",
    }
}
