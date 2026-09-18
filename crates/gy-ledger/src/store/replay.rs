//! Replaying the event log: node state and history (D-82).
use super::{Actor, Error, HistoryEntry, Result, log, snapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

/// The node's scope field, so the store renames it without a string key.
#[derive(Serialize, Deserialize)]
struct Scoped {
    scope: String,
    #[serde(flatten)]
    rest: serde_json::Map<String, Value>,
}

/// The node's `created` field, so replay can normalize an old date-only value
/// without reading an attribute by string key (n-b6b6).
#[derive(Serialize, Deserialize)]
struct Dated {
    created: String,
    #[serde(flatten)]
    rest: serde_json::Map<String, Value>,
}

/// A node value whose old `YYYY-MM-DD` created is raised to a UTC instant.
/// Every event ever written keeps its own form, so this runs on every read,
/// whatever the ledger's format version (n-b6b6).
fn raised_created(value: &Value) -> Value {
    let Ok(dated) = serde_json::from_value::<Dated>(value.clone()) else {
        return value.clone();
    };
    if dated.created.len() != 10 {
        return value.clone();
    }
    let raised = Dated {
        created: format!("{}T00:00:00Z", dated.created),
        rest: dated.rest,
    };
    serde_json::to_value(&raised).unwrap_or_else(|_| value.clone())
}

/// The scope a stored node carries, or `None` when it is not a node.
pub(super) fn scope_of(value: &Value) -> Option<String> {
    serde_json::from_value::<Scoped>(value.clone())
        .ok()
        .map(|scoped| scoped.scope)
}

/// Move one stored node to another scope; returns whether it carried the old
/// scope.
pub(super) fn rename_value(value: &mut Value, from: &str, to: &str) -> bool {
    let Ok(mut scoped) = serde_json::from_value::<Scoped>(value.clone()) else {
        return false;
    };
    if scoped.scope != from {
        return false;
    }
    scoped.scope = to.to_string();
    match serde_json::to_value(&scoped) {
        Ok(updated) => {
            *value = updated;
            true
        }
        Err(_) => false,
    }
}

/// Apply an event's changes to a node map. A deletion removes, another node
/// change inserts the carried value, and a scope rename repoints every node
/// that carries the old scope.
pub fn apply(nodes: &mut BTreeMap<String, Value>, event: &log::Event) {
    for change in &event.changes {
        match change {
            log::Change::Created { node, value } | log::Change::Updated { node, value } => {
                nodes.insert(node.clone(), raised_created(value));
            }
            log::Change::Deleted { node, .. } => {
                nodes.remove(node);
            }
            log::Change::ScopeRenamed { from, to, .. } => {
                for value in nodes.values_mut() {
                    rename_value(value, from, to);
                }
            }
        }
    }
}

/// Rebuild the history from every event, in order (AC-46). A scope rename is
/// one entry with an empty node and the kind `scope-renamed`.
pub fn history(events: &[log::Event]) -> Result<Vec<HistoryEntry>> {
    let mut history = Vec::new();
    for event in events {
        for change in &event.changes {
            let (node, what) = match change {
                log::Change::Created { node, .. } => (node.clone(), "created".to_string()),
                log::Change::Updated { node, .. } => (node.clone(), "updated".to_string()),
                log::Change::Deleted { node, .. } => (node.clone(), "deleted".to_string()),
                log::Change::ScopeRenamed { from, to, nodes } => {
                    (String::new(), super::rename_line(from, to, *nodes))
                }
            };
            history.push(HistoryEntry {
                seq: event.seq,
                at: event.at,
                actor: Actor::new(event.actor.clone())?,
                node,
                what,
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
