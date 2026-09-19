//! The scope index README: heading, reading, lists, history, and diagnostics
//! (d-7c64).
use super::super::derive::need_state;
use super::{FileEntry, diagnostics, history, path, plural, reference};
use crate::model::{Node, NodeData, NodeKind};
use crate::ops::repository::{Repository, Result, Store};
use std::fmt::Write;

pub(super) fn index<S: Store>(
    repository: &Repository<S>,
    all: &[Node],
    scope: &str,
    since: Option<u64>,
    seq: u64,
    writer: &str,
    location: &str,
) -> Result<FileEntry> {
    let mut out = header(scope, seq, since, writer, location);
    out.push_str("\n## How to read\n\n");
    out.push_str(reading());
    out.push_str("\n## Lists\n\n");
    out.push_str(&lists(all, scope));
    out.push_str("\n## History\n\n");
    out.push_str(&history::history(repository, all, scope, since)?);
    out.push_str("\n## Diagnostics\n\n");
    out.push_str(&diagnostics::diagnostics(repository, Some(scope))?);
    Ok(FileEntry {
        path: "README.md".to_string(),
        text: out,
    })
}

fn header(scope: &str, seq: u64, since: Option<u64>, writer: &str, location: &str) -> String {
    let generated = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let since = since.map_or("(from the start)".to_string(), |since| since.to_string());
    let writer = if writer.trim().is_empty() {
        String::new()
    } else {
        format!("- Writer: {writer}\n")
    };
    format!(
        "# gy publication — {scope}\n\n\
         - Generated: {generated}\n\
         - seq: {seq}\n\
         - scope: {scope}\n\
         - since: {since}\n\
         {writer}- Canonical store: {location}\n"
    )
}

/// The per-kind list, each line a link to the node's file and its state.
fn lists(all: &[Node], scope: &str) -> String {
    let mut out = String::new();
    for kind in NodeKind::ALL {
        let _ = writeln!(out, "### {}", plural(kind));
        let mut nodes: Vec<&Node> = all
            .iter()
            .filter(|node| node.kind() == kind && node.scope() == scope)
            .collect();
        nodes.sort_by_key(|node| (node.created().to_string(), node.id().to_string()));
        if nodes.is_empty() {
            out.push_str("- none\n");
            continue;
        }
        for node in nodes {
            match state(node, all) {
                Some(state) => {
                    let _ = writeln!(out, "- [{}]({}) — {state}", reference(node), path(node));
                }
                None => {
                    let _ = writeln!(out, "- [{}]({})", reference(node), path(node));
                }
            }
        }
    }
    out
}

/// A node's state as a list shows it; a decision has none.
fn state(node: &Node, all: &[Node]) -> Option<String> {
    match node.data() {
        NodeData::Need(_) => Some(need_state(node, all).name().to_string()),
        NodeData::Question(data) => Some(
            if data.closure.is_some() {
                "closed"
            } else {
                "open"
            }
            .to_string(),
        ),
        NodeData::Requirement(data) => Some(data.state.name().to_string()),
        NodeData::Criterion(data) => Some(
            if data.satisfied {
                "satisfied"
            } else {
                "unsatisfied"
            }
            .to_string(),
        ),
        NodeData::Decision(_) => None,
    }
}

/// What a reader needs to read the rest without the ledger.
fn reading() -> &'static str {
    "Node kinds (5):\n\
     - need (n-…): work still to do. It targets criteria.\n\
     - question (q-…): something not yet decided. It has a decider and two or more options.\n\
     - decision (d-…): a decision, with the conditions under which it holds (decision_scope).\n\
     - requirement (r-…): what will change, filed for approval. It may carry an external reference (ref).\n\
     - criterion (ac-…): an acceptance criterion. The unit that counts a need's progress.\n\n\
     Requirement states (4):\n\
     - filed: filed. The design is not yet approved.\n\
     - approved: confirmed. The design was approved.\n\
     - done: done. The ground for shipping is recorded.\n\
     - cancelled: cancelled. Closed for a reason other than completion.\n\n\
     Relations (12) and their direction:\n\
     - closes: question → decision.\n\
     - narrows / widens / supersedes / completes: decision → decision.\n\
     - targets: need / requirement → criterion.\n\
     - spawned-by: need → decision.\n\
     - filed-as: need → requirement.\n\
     - depends-on: need → need.\n\
     - relies-on: requirement → decision.\n\
     - raised: requirement → question.\n\
     - waits-on: need → question / requirement.\n\
     An edge is stored only on its source node; the reverse direction is derived. This document shows both directions.\n\n\
     How a question closes:\n\
     - fact: closed on a fact. - decision: closed on a decision. - non-decision: closed without a decision.\n\n\
     IDs and aliases:\n\
     - An ID is the kind's prefix and a short hash (e.g. n-3f9a). An older ID, if any, is listed with it as an alias (e.g. D-78).\n\
     - Every reference is written \"ID (alias) title\". A requirement also has a ref (an external reference).\n\n\
     History fields:\n\
     - seq: the write's serial number (a point in time). - when / writer: when and who (GY_ACTOR).\n\
     - node: the target ID and title. - what: created / updated / deleted.\n\
     - why: the operation's name and target. - source: the ground or URL passed to the operation.\n"
}
