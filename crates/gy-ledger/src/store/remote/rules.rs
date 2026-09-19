//! Typed node/link reads and `rejected.jsonl` for the rebase (n-ecbf 2B).
use super::super::{Error, FormatVersion, Result, log, replay};
use super::git;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// An error with the way out on its second line (n-94bb 3B).
pub(super) fn advice(error: Error, method: &str) -> Error {
    Error::invalid(format!("{}\n{method}", error.message))
}

/// Refuse a remote whose ledger format is newer than this build reads
/// (n-94bb 3B). A remote without a `format` file yet is fine.
pub(super) fn check_remote_format(ledger: &Path, upstream: &str) -> Result<()> {
    let Some(text) = git::show(ledger, upstream, super::super::format::FILE) else {
        return Ok(());
    };
    let version: u32 = text
        .trim()
        .parse()
        .map_err(|_| Error::invalid("the remote format file is not a version number"))?;
    if version > FormatVersion::CURRENT.0 {
        return Err(Error::invalid(format!(
            "the remote ledger is format {version}; this build reads up to {}\nupdate gy (this build never raises the remote's format)",
            FormatVersion::CURRENT.0
        )));
    }
    Ok(())
}
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
/// The identity of a write across a rebase: everything but the sequence, which
/// a rebase may change (n-6b44).
pub(super) fn key(event: &log::Event) -> String {
    let mut copy = event.clone();
    copy.seq = 0;
    serde_json::to_string(&copy).unwrap_or_default()
}

/// The remote events in `(committed, ..]` whose content is one of the copy's
/// uncommitted writes. They already landed, so a rebase counts them as pushed
/// instead of refusing them as the remote's change (n-6b44).
pub(super) fn landed(
    remote: &[log::Event],
    committed: u64,
    pending: &[log::Event],
) -> BTreeSet<String> {
    let wanted: BTreeSet<String> = pending.iter().map(key).collect();
    remote
        .iter()
        .filter(|event| event.seq > committed && wanted.contains(&key(event)))
        .map(key)
        .collect()
}

pub(super) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
