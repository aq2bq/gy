//! Opening the ledger as of a sequence: the log replayed up to `seq` into a
//! memory store, so a read can answer for any point (n-10e1, D-82). Nothing is
//! written and the snapshot is not used.
use super::{Actor, MemoryStore, Result, format, log, replay};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

/// Open the ledger at `dir` as it stood after the write with sequence `seq`:
/// the events up to it replayed, with the history of that range. A `seq` at or
/// past the end gives the same nodes and history as `FileStore::open_with`.
pub fn open_at(
    dir: &Path,
    seq: u64,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<MemoryStore> {
    let version = format::read(dir)?;
    let actor = Actor::from_lookup(lookup)?;
    let (events, _) = log::read(dir)?;
    let retired = retirements(&events);
    let events: Vec<log::Event> = events
        .into_iter()
        .filter(|event| event.seq <= seq)
        .collect();
    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    for event in &events {
        replay::apply(&mut nodes, event);
    }
    // Retirement applies to every read, including a point before the write.
    for (id, seq) in retired {
        if let Some(value) = nodes.get_mut(&id) {
            super::retirement::set(value, seq);
        }
    }
    let history = replay::history(&events)?;
    MemoryStore::from_replay(version, actor, nodes, history)
}

fn retirements(events: &[log::Event]) -> BTreeMap<String, Option<u64>> {
    let mut retired = BTreeMap::new();
    for event in events {
        for change in &event.changes {
            if let log::Change::Retired { nodes, revoke } = change {
                let seq = if *revoke { None } else { Some(event.seq) };
                retired.extend(nodes.iter().map(|id| (id.clone(), seq)));
            }
        }
    }
    retired
}
