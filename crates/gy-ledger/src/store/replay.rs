//! Replaying the event log: node state and history (D-82).
use super::{Actor, Error, HistoryEntry, Result, log, snapshot};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

/// Apply an event's changes to a node map: deleted removes, anything else
/// inserts the carried value.
pub fn apply(nodes: &mut BTreeMap<String, Value>, event: &log::Event) {
    for change in &event.changes {
        if change.change == "deleted" {
            nodes.remove(&change.node);
        } else {
            nodes.insert(change.node.clone(), change.value.clone());
        }
    }
}

/// Rebuild the history from every event, in order (AC-46).
pub fn history(events: &[log::Event]) -> Result<Vec<HistoryEntry>> {
    let mut history = Vec::new();
    for event in events {
        for change in &event.changes {
            history.push(HistoryEntry {
                seq: event.seq,
                at: event.at,
                actor: Actor::new(event.actor.clone())?,
                node: change.node.clone(),
                what: change.change.clone(),
                why: event.why.clone(),
                source: event.source.clone(),
            });
        }
    }
    Ok(history)
}

/// Rebuild the nodes from the snapshot plus the events after it, or from the
/// whole log. Returns the nodes, the events replayed, and whether the snapshot
/// must be rewritten.
pub fn load_nodes(dir: &Path, events: &[log::Event]) -> (BTreeMap<String, Value>, usize, bool) {
    let last = events.last().map_or(0, |event| event.seq);
    match snapshot::read(dir) {
        Some((seq, mut nodes)) if seq <= last => {
            let mut replayed = 0;
            for event in events.iter().filter(|event| event.seq > seq) {
                apply(&mut nodes, event);
                replayed += 1;
            }
            (nodes, replayed, false)
        }
        _ => {
            let mut nodes = BTreeMap::new();
            for event in events {
                apply(&mut nodes, event);
            }
            (nodes, events.len(), true)
        }
    }
}

/// Serialize a value for staging or storing.
pub fn encode(value: &Value) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| Error::invalid(error.to_string()))
}
