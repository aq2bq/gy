//! The refused writes recorded in `rejected.jsonl`: the type, its reads and
//! writes, and the notices a rebase leaves for the writer (n-38e3).
use super::super::{Error, Result, log};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rejected {
    pub at: u64,
    pub event: log::Event,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<RejectedBy>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedBy {
    pub seq: u64,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
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
fn read_rejected(ledger: &Path) -> Vec<Rejected> {
    std::fs::read_to_string(ledger.join("rejected.jsonl"))
        .map(|text| {
            text.lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect()
        })
        .unwrap_or_default()
}
pub fn rejected_notices(ledger: &Path) -> Vec<String> {
    read_rejected(ledger)
        .iter()
        .map(|rejected| {
            format!(
                "notice: your write seq {} ({}) did not land: {}; redo it against the current ledger if it still applies",
                rejected.event.seq, rejected.event.why, rejected.reason
            )
        })
        .collect()
}
pub fn clear_rejected(ledger: &Path, event: &log::Event) -> Result<()> {
    let Ok(text) = std::fs::read_to_string(ledger.join("rejected.jsonl")) else {
        return Ok(());
    };
    let mine = named_nodes(event);
    let mut kept = Vec::new();
    let mut removed = false;
    for line in text.lines() {
        match serde_json::from_str::<Rejected>(line) {
            Ok(rejected)
                if rejected.event.actor == event.actor
                    && rejected.event.by == event.by
                    && named_nodes(&rejected.event)
                        .iter()
                        .any(|node| mine.contains(node)) =>
            {
                removed = true;
            }
            _ => kept.push(line),
        }
    }
    if !removed {
        return Ok(());
    }
    let mut out = kept.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    let temporary = ledger.join(format!(".rejected.{}.tmp", std::process::id()));
    std::fs::write(&temporary, out)?;
    std::fs::rename(&temporary, ledger.join("rejected.jsonl"))?;
    Ok(())
}
fn named_nodes(event: &log::Event) -> BTreeSet<String> {
    event
        .changes
        .iter()
        .filter_map(|change| match change {
            log::Change::Created { node, .. }
            | log::Change::Updated { node, .. }
            | log::Change::Deleted { node, .. } => Some(node.clone()),
            log::Change::ScopeRenamed { .. } => None,
        })
        .collect()
}
