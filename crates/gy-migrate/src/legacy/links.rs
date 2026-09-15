//! Reading a 0.4 node's edges: the from-side keys and their `{id, mark}` form.
use gy_core::text;

/// The forward relationship keys a 0.4 node stores (`measured-by` becomes a
/// waits-on edge for a gate).
pub(super) const RELATIONS: &[&str] = &[
    "targets",
    "spawned-by",
    "filed-as",
    "depends-on",
    "relies-on",
    "raised",
    "closes",
    "narrows",
    "widens",
    "supersedes",
    "completes",
    "measured-by",
];

/// One 0.4 edge: the target's old id and, for narrows / supersedes, its mark.
#[derive(Debug, Clone)]
pub struct LegacyLink {
    pub id: String,
    pub mark: Option<String>,
}

/// The edges carried by one attribute: a list of ids, a list of `{id, mark}`
/// objects, or a single one of either.
pub(super) fn edges_of(value: &serde_json::Value) -> Vec<LegacyLink> {
    match value {
        serde_json::Value::Array(items) => items.iter().flat_map(edges_of).collect(),
        serde_json::Value::Object(map) => match map.get("id") {
            Some(id) => vec![LegacyLink {
                id: text(id),
                mark: map.get("mark").map(text).filter(|mark| !mark.is_empty()),
            }],
            None => Vec::new(),
        },
        _ => vec![LegacyLink {
            id: text(value),
            mark: None,
        }],
    }
}
