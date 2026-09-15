//! The free attributes a 0.4 node carries into the new ledger: everything that
//! is not a typed field, an edge, a dropped name, or a frozen record (n-9198).
use super::links::RELATIONS;
use super::node::{DROPPED, LegacyNode, RECORDS};

/// Attributes that are identity or shared fields, never copied verbatim.
const SHARED: &[&str] = &["id", "type", "title", "scope", "created", "status"];

/// Attributes the transfer turns into edges elsewhere.
const HANDLED: &[&str] = &["waiting-on", "belongs-to"];

/// The other side of a 0.4 relation, also an edge: the new model derives it.
const REVERSE: &[&str] = &[
    "closed-by",
    "narrowed-by",
    "widened-by",
    "superseded-by",
    "completed-by",
    "targeted-by",
    "spawns",
    "files",
    "filed-from",
    "depended-on-by",
    "needed-by",
    "relied-on-by",
    "raised-by",
    "awaited-by",
];

/// The attributes to carry as free attributes.
pub(crate) fn attributes(node: &LegacyNode) -> Vec<(String, String)> {
    node.values
        .iter()
        .filter(|(name, _)| !reserved(node, name))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

/// A name the transfer reads elsewhere, so it is never copied verbatim.
fn reserved(node: &LegacyNode, name: &str) -> bool {
    SHARED.contains(&name)
        || HANDLED.contains(&name)
        || RELATIONS.contains(&name)
        || REVERSE.contains(&name)
        || DROPPED.contains(&name)
        || RECORDS.contains(&name)
        || name.starts_with("production_")
        || name.starts_with("pr_")
        || typed_extra(node, name)
}

/// The typed fields a kind reads from its attributes.
fn typed_extra(node: &LegacyNode, name: &str) -> bool {
    match node.new_kind() {
        "decision" => name == "decision_scope",
        "question" => matches!(
            name,
            "decider" | "options" | "closed_by" | "closure_note" | "closed_at" | "evidence"
        ),
        "criterion" => matches!(name, "satisfied" | "satisfied_at" | "evidence"),
        "need" => matches!(name, "dropped_by" | "dropped_reason"),
        _ => false,
    }
}

/// One attribute as text: an array joins its elements by newline, an object is
/// JSON, anything else is its own text.
pub(super) fn render(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .map(gy_core::text)
            .collect::<Vec<_>>()
            .join("\n"),
        serde_json::Value::Object(_) => value.to_string(),
        _ => gy_core::text(value),
    }
}
