//! Rebase uncommitted writes onto the remote's new lines (n-ecbf 2B, ac-08cc).
use super::super::{Result, log, replay};
use super::report::{Pulled, Range, Sync};
use super::rules::{now, parse, targets, touched, write_rejected};
use super::{git, sync::push_writes, sync::rebuild_snapshot, sync::writers};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
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
type Cause = (u64, String, Option<String>);
pub(super) fn rebase(
    ledger: &Path,
    branch: &str,
    upstream: &str,
    committed: u64,
    report: &mut Sync,
) -> Result<()> {
    let (remote_text, remote, pending) = inputs(ledger, upstream, committed)?;
    let (mut nodes, changed, removed, causes, remote_seq) = remote_changes(&remote, committed);
    let (accepted, rejects) = place(&mut nodes, pending, &changed, &removed, &causes, remote_seq);
    git::reset_hard(ledger, upstream)?;
    let mut text = remote_text;
    for event in &accepted {
        text.push_str(&log::line(event)?);
    }
    log::replace(ledger, &text)?;
    rebuild_snapshot(ledger)?;
    report.pulled = Some(Pulled {
        from: committed,
        to: remote_seq,
        writers: writers(ledger, committed, remote_seq)?,
    });
    if !accepted.is_empty() {
        report.rebased = Some(Range {
            from: remote_seq + 1,
            to: remote_seq + accepted.len() as u64,
        });
        push_writes(ledger, branch, remote_seq, report)?;
    }
    if !rejects.is_empty() {
        write_rejected(ledger, &rejects)?;
        report.rejected = Some(rejects.len());
    }
    report.seq = log::read(ledger)?.0.last().map_or(0, |event| event.seq);
    Ok(())
}
fn inputs(
    ledger: &Path,
    upstream: &str,
    committed: u64,
) -> Result<(String, Vec<log::Event>, Vec<log::Event>)> {
    let remote_text = git::show(ledger, upstream, log::FILE).unwrap_or_default();
    let remote = parse(&remote_text)?;
    let pending = log::read(ledger)?
        .0
        .into_iter()
        .filter(|event| event.seq > committed)
        .collect();
    Ok((remote_text, remote, pending))
}
#[allow(clippy::type_complexity)]
fn remote_changes(
    remote: &[log::Event],
    committed: u64,
) -> (
    BTreeMap<String, Value>,
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeMap<String, Cause>,
    u64,
) {
    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    let mut changed = BTreeSet::new();
    let mut removed = BTreeSet::new();
    let mut causes: BTreeMap<String, Cause> = BTreeMap::new();
    let mut remote_seq = committed;
    for event in remote {
        if event.seq <= committed {
            replay::apply(&mut nodes, event);
            continue;
        }
        remote_seq = event.seq;
        for node in touched(&nodes, event) {
            changed.insert(node.clone());
            causes
                .entry(node)
                .or_insert_with(|| (event.seq, event.actor.clone(), event.by.clone()));
        }
        for change in &event.changes {
            if let log::Change::Deleted { node, .. } = change {
                removed.insert(node.clone());
            }
        }
        replay::apply(&mut nodes, event);
    }
    (nodes, changed, removed, causes, remote_seq)
}
fn place(
    nodes: &mut BTreeMap<String, Value>,
    pending: Vec<log::Event>,
    changed: &BTreeSet<String>,
    removed: &BTreeSet<String>,
    causes: &BTreeMap<String, Cause>,
    remote_seq: u64,
) -> (Vec<log::Event>, Vec<Rejected>) {
    let mut accepted = Vec::new();
    let mut rejects = Vec::new();
    let mut rejected_created: BTreeSet<String> = BTreeSet::new();
    let mut next = remote_seq + 1;
    for event in pending {
        if let Some(rejected) =
            rejection(nodes, changed, removed, &rejected_created, causes, &event)
        {
            for change in &rejected.event.changes {
                if let log::Change::Created { node, .. } = change {
                    rejected_created.insert(node.clone());
                }
            }
            rejects.push(rejected);
        } else {
            let mut placed = event;
            placed.seq = next;
            next += 1;
            replay::apply(nodes, &placed);
            accepted.push(placed);
        }
    }
    (accepted, rejects)
}
fn rejection(
    nodes: &BTreeMap<String, Value>,
    changed: &BTreeSet<String>,
    removed: &BTreeSet<String>,
    rejected_created: &BTreeSet<String>,
    causes: &BTreeMap<String, Cause>,
    event: &log::Event,
) -> Option<Rejected> {
    let touched = touched(nodes, event);
    let reason = judge(nodes, changed, removed, rejected_created, event, &touched)?;
    let by = touched
        .iter()
        .find_map(|node| causes.get(node))
        .map(|cause| RejectedBy {
            seq: cause.0,
            actor: cause.1.clone(),
            by: cause.2.clone(),
        });
    Some(Rejected {
        at: now(),
        event: event.clone(),
        reason,
        by,
    })
}
fn judge(
    nodes: &BTreeMap<String, Value>,
    changed: &BTreeSet<String>,
    removed: &BTreeSet<String>,
    rejected_created: &BTreeSet<String>,
    event: &log::Event,
    touched: &[String],
) -> Option<String> {
    for node in touched {
        if removed.contains(node) {
            return Some(format!("the remote deleted {node}"));
        }
        if changed.contains(node) {
            return Some(format!("the remote changed {node}"));
        }
    }
    for change in &event.changes {
        if let log::Change::Created { node, .. } = change {
            if nodes.contains_key(node) {
                return Some(format!("{node} already exists in the remote's ledger"));
            }
        }
        let value = match change {
            log::Change::Created { value, .. } | log::Change::Updated { value, .. } => value,
            _ => continue,
        };
        for target in targets(value) {
            if rejected_created.contains(&target) {
                return Some(format!(
                    "it points at {target}, which a refused write created"
                ));
            }
            if !nodes.contains_key(&target) {
                return Some(format!("it points at {target}, which is not in the ledger"));
            }
        }
    }
    None
}
