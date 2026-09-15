//! The record: every node in range, verbatim, in a deterministic order
//! (d-edb0). Nothing is summarized or reworded, and every reference carries its
//! target's title so the document stands alone.
use super::super::show::{Shown, show};
use super::{in_scope, reference};
use crate::model::{ClosedBy, Need, Node, NodeData, NodeKind, Question, Requirement};
use crate::ops::repository::{Repository, Result, Store};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

pub(super) fn nodes<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
    since: Option<u64>,
) -> Result<String> {
    let all = repository.all()?;
    let targets: BTreeMap<String, &Node> = all
        .iter()
        .map(|node| (node.id().to_string(), node))
        .collect();
    let ids: Vec<String> = ordered(repository, &all, scope, since)
        .iter()
        .map(|node| node.id().to_string())
        .collect();

    let mut out = String::new();
    for shown in show(repository, &ids, true)? {
        out.push_str(&block(&shown, &targets));
    }
    if out.is_empty() {
        out.push_str("（記録は無し）\n");
    }
    Ok(out)
}

/// Scope, then kind, then creation date and ID: one fixed order, so two
/// publications can be diffed.
fn ordered<'a, S: Store>(
    repository: &Repository<S>,
    all: &'a [Node],
    scope: Option<&str>,
    since: Option<u64>,
) -> Vec<&'a Node> {
    let changed = changed_ids(repository, since);
    let mut nodes: Vec<&Node> = all
        .iter()
        .filter(|node| in_scope(node, scope))
        .filter(|node| since.is_none() || changed.contains(&node.id().to_string()))
        .collect();
    nodes.sort_by_key(|node| key(node));
    nodes
}

fn changed_ids<S: Store>(repository: &Repository<S>, since: Option<u64>) -> BTreeSet<String> {
    let Some(since) = since else {
        return BTreeSet::new();
    };
    repository
        .store()
        .history()
        .iter()
        .filter(|entry| entry.seq > since)
        .map(|entry| entry.node.clone())
        .collect()
}

fn key(node: &Node) -> (String, usize, String, String) {
    (
        node.scope().to_string(),
        NodeKind::ALL
            .iter()
            .position(|kind| *kind == node.kind())
            .unwrap_or(0),
        node.created().to_string(),
        node.id().to_string(),
    )
}

fn block(shown: &Shown, targets: &BTreeMap<String, &Node>) -> String {
    let mut out = format!("### {} {}\n", heading(shown), shown.title);
    out.push_str(&shown.verbatim());
    if let Some(scope) = &shown.scope {
        let _ = writeln!(out, "scope: {scope}");
    }
    if let Some(created) = &shown.created {
        let _ = writeln!(out, "created: {created}");
    }
    for (name, value) in &shown.attributes {
        let _ = writeln!(out, "{name}: {value}");
    }
    for edge in &shown.edges {
        let target = targets
            .get(&edge.to)
            .map(|node| reference(node))
            .unwrap_or_else(|| edge.to.clone());
        let mark = edge
            .mark
            .as_deref()
            .map(|mark| format!("（mark: {mark}）"))
            .unwrap_or_default();
        let _ = writeln!(out, "  {} {target}{mark}", edge.name);
    }
    if !shown.body.is_empty() {
        let _ = writeln!(out, "{}", shown.body);
    }
    records(&shown.data, &mut out);
    out.push('\n');
    out
}

/// The id with its old aliases and, for a requirement, its reference (D-62).
fn heading(shown: &Shown) -> String {
    let mut extra = shown.aliases.clone();
    if let Some(reference) = &shown.reference {
        extra.push(reference.clone());
    }
    if extra.is_empty() {
        shown.id.clone()
    } else {
        format!("{} ({})", shown.id, extra.join(", "))
    }
}

fn records(data: &NodeData, out: &mut String) {
    match data {
        NodeData::Requirement(data) => requirement_records(data, out),
        NodeData::Question(data) => question_records(data, out),
        NodeData::Need(data) => need_records(data, out),
        _ => {}
    }
}

fn requirement_records(data: &Requirement, out: &mut String) {
    if let Some(approval) = &data.approval {
        let _ = writeln!(
            out,
            "承認: design={}, heard_by={}, evidence={}, at={}",
            approval.design, approval.heard_by, approval.evidence, approval.at
        );
    }
    for revision in &data.revisions {
        let _ = writeln!(
            out,
            "改訂: {}（出典: {}、{}）",
            revision.reason, revision.source, revision.at
        );
    }
    if let Some(completion) = &data.completion {
        let _ = writeln!(out, "完了: {}（{}）", completion.evidence, completion.at);
    }
    if let Some(cancellation) = &data.cancellation {
        let _ = writeln!(
            out,
            "中止: {}（出典: {}、{}）",
            cancellation.reason, cancellation.source, cancellation.at
        );
    }
}

fn question_records(data: &Question, out: &mut String) {
    let evidence = data.evidence.clone().unwrap_or_default();
    let line = match data.closure {
        Some(crate::model::Closure::Fact) => format!("閉じ方: 事実（{evidence}）"),
        Some(crate::model::Closure::Decision) => format!("閉じ方: 決定（{evidence}）"),
        Some(crate::model::Closure::NonDecision) => format!("閉じ方: 決定を伴わない（{evidence}）"),
        None => "閉じ方: 開いている".to_string(),
    };
    let _ = writeln!(out, "{line}");
}

fn need_records(data: &Need, out: &mut String) {
    if let Some(closed) = &data.closed {
        let by = match closed.by {
            ClosedBy::Fact => "事実",
            ClosedBy::External => "外部",
        };
        let _ = writeln!(out, "閉じた理由: {by}（{}）", closed.evidence);
    }
}
