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

/// One requirement record's `at`, so replay can normalize it without a string
/// key (n-86cc). The other fields ride along in `rest`.
#[derive(Serialize, Deserialize)]
struct Instant {
    at: String,
    #[serde(flatten)]
    rest: serde_json::Map<String, Value>,
}

/// A requirement's records, as the stored value carries them (n-86cc).
#[derive(Serialize, Deserialize)]
struct RequirementShape {
    #[serde(default)]
    approval: Option<Instant>,
    #[serde(default)]
    revisions: Vec<Instant>,
    #[serde(default)]
    completion: Option<Instant>,
    #[serde(default)]
    cancellation: Option<Instant>,
    #[serde(flatten)]
    rest: serde_json::Map<String, Value>,
}

/// A criterion's `satisfied_at`, as the stored value carries it (n-86cc).
#[derive(Serialize, Deserialize)]
struct CriterionShape {
    #[serde(default)]
    satisfied_at: Option<String>,
    #[serde(flatten)]
    rest: serde_json::Map<String, Value>,
}

/// The five kind-specific payloads, typed only where a date lives.
#[derive(Serialize, Deserialize)]
enum DataShape {
    Need(Value),
    Question(Value),
    Decision(Value),
    Requirement(RequirementShape),
    Criterion(CriterionShape),
}

/// A stored node with every instant bearing a date reachable by name.
#[derive(Serialize, Deserialize)]
struct NodeShape {
    created: String,
    data: DataShape,
    #[serde(flatten)]
    rest: serde_json::Map<String, Value>,
}

/// A bare `YYYY-MM-DD` raised to the UTC instant it stood for; another form is
/// left alone.
fn raise(text: &str) -> String {
    if text.len() == 10 {
        format!("{text}T00:00:00Z")
    } else {
        text.to_string()
    }
}

/// Raise one requirement record's `at` in place; whether it changed.
fn raise_instant(instant: &mut Instant) -> bool {
    if instant.at.len() == 10 {
        instant.at = raise(&instant.at);
        true
    } else {
        false
    }
}

/// A node value whose old `YYYY-MM-DD` instants are raised to UTC. Events ever
/// written keep their own form, so this runs on every read, whatever the
/// ledger's format version (n-b6b6, n-86cc).
fn raise_dates(value: &Value) -> Value {
    let Ok(mut node) = serde_json::from_value::<NodeShape>(value.clone()) else {
        return value.clone();
    };
    let mut changed = node.created.len() == 10;
    if changed {
        node.created = raise(&node.created);
    }
    match &mut node.data {
        DataShape::Criterion(data) => {
            if let Some(at) = data.satisfied_at.as_mut() {
                if at.len() == 10 {
                    *at = raise(at);
                    changed = true;
                }
            }
        }
        DataShape::Requirement(data) => {
            for instant in data.approval.iter_mut() {
                changed |= raise_instant(instant);
            }
            for instant in data.completion.iter_mut() {
                changed |= raise_instant(instant);
            }
            for instant in data.cancellation.iter_mut() {
                changed |= raise_instant(instant);
            }
            for revision in &mut data.revisions {
                changed |= raise_instant(revision);
            }
        }
        DataShape::Need(_) | DataShape::Question(_) | DataShape::Decision(_) => {}
    }
    if !changed {
        return value.clone();
    }
    serde_json::to_value(&node).unwrap_or_else(|_| value.clone())
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
                nodes.insert(node.clone(), raise_dates(value));
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
                by: event.by.clone(),
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
            for value in nodes.values_mut() {
                let raised = raise_dates(value);
                *value = raised;
            }
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
