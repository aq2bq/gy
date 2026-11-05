//! The retirement marker in stored JSON. The payload is never removed.
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct Marked {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retired: Option<u64>,
    #[serde(flatten)]
    rest: serde_json::Map<String, Value>,
}
pub(super) fn set(value: &mut Value, seq: Option<u64>) {
    if let Ok(mut marked) = serde_json::from_value::<Marked>(value.clone()) {
        marked.retired = seq;
        if let Ok(updated) = serde_json::to_value(marked) {
            *value = updated;
        }
    }
}
pub(super) fn line(nodes: &[String], revoke: bool) -> String {
    format!(
        "{} {}",
        if revoke {
            "retirement-revoked"
        } else {
            "retired"
        },
        nodes.join(" ")
    )
}
