//! Typed node/link reads and `rejected.jsonl` for the rebase (n-ecbf 2B).
use super::super::{Error, Result, log, replay};
use super::rebase::Rejected;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
#[derive(Deserialize)]
struct NodeLinks {
    #[serde(default)]
    links: Vec<EdgeShape>,
}
#[derive(Deserialize)]
struct EdgeShape {
    to: IdShape,
}
#[derive(Deserialize)]
struct IdShape {
    kind: String,
    hash: String,
}
pub(super) fn targets(value: &Value) -> Vec<String> {
    let Ok(node) = serde_json::from_value::<NodeLinks>(value.clone()) else {
        return Vec::new();
    };
    node.links
        .iter()
        .filter_map(|link| Some(format!("{}-{}", prefix(&link.to.kind)?, link.to.hash)))
        .collect()
}
pub(super) fn prefix(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "Need" => "n",
        "Question" => "q",
        "Decision" => "d",
        "Requirement" => "r",
        "Criterion" => "ac",
        _ => return None,
    })
}
pub(super) fn touched(nodes: &BTreeMap<String, Value>, event: &log::Event) -> Vec<String> {
    let mut out = Vec::new();
    for change in &event.changes {
        match change {
            log::Change::Created { node, .. }
            | log::Change::Updated { node, .. }
            | log::Change::Deleted { node, .. } => out.push(node.clone()),
            log::Change::ScopeRenamed { from, .. } => {
                for (id, value) in nodes {
                    if replay::scope_of(value).as_deref() == Some(from) {
                        out.push(id.clone());
                    }
                }
            }
        }
    }
    out
}
pub(super) fn parse(text: &str) -> Result<Vec<log::Event>> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line)
                .map_err(|_| Error::invalid("a committed event is not an event"))
        })
        .collect()
}
pub(super) fn write_rejected(ledger: &Path, rejects: &[Rejected]) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger.join("rejected.jsonl"))?;
    for rejected in rejects {
        let mut text =
            serde_json::to_string(rejected).map_err(|e| Error::invalid(e.to_string()))?;
        text.push('\n');
        file.write_all(text.as_bytes())?;
    }
    Ok(())
}
pub(super) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
