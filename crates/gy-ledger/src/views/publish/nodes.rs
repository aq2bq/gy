//! The node files: one file per node, named for its kind and ID (d-7c64).
use super::super::show::{EdgeLine, Shown, show};
use super::{FileEntry, path, reference};
use crate::model::{ClosedBy, Closure, Need, Node, NodeData, NodeKind, Question, Requirement};
use crate::ops::repository::{Repository, Result, Store};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// The lineage relations a decision prints first, one per line.
const LINEAGE: &[&str] = &[
    "supersedes",
    "superseded-by",
    "narrows",
    "narrowed-by",
    "widens",
    "completes",
];

/// Every node file of one scope, in the fixed order kind, created, ID.
pub(super) fn files<S: Store>(
    repository: &Repository<S>,
    all: &[Node],
    scope: &str,
    since: Option<u64>,
) -> Result<Vec<FileEntry>> {
    let targets: BTreeMap<String, &Node> = all
        .iter()
        .map(|node| (node.id().to_string(), node))
        .collect();
    let changed = changed_ids(repository, since);
    let mut nodes: Vec<&Node> = all
        .iter()
        .filter(|node| node.scope() == scope)
        .filter(|node| since.is_none() || changed.contains(&node.id().to_string()))
        .collect();
    nodes.sort_by_key(|node| key(node));

    let mut files = Vec::new();
    for node in nodes {
        let shown = show(repository, &[node.id().to_string()], true)?;
        if let Some(shown) = shown.first() {
            files.push(FileEntry {
                path: path(node),
                text: text(shown, &targets),
            });
        }
    }
    Ok(files)
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

fn key(node: &Node) -> (usize, String, String) {
    (
        NodeKind::ALL
            .iter()
            .position(|kind| *kind == node.kind())
            .unwrap_or(0),
        node.created().to_string(),
        node.id().to_string(),
    )
}

fn text(shown: &Shown, targets: &BTreeMap<String, &Node>) -> String {
    let mut out = format!("# {} {}\n\n", heading(shown), shown.title);
    if shown.kind == NodeKind::Decision {
        out.push_str(&relations(shown, targets));
    }
    out.push_str(&rest(shown, targets));
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

/// The decision's lineage edges first, with the other side's title (d-7c64).
fn relations(shown: &Shown, targets: &BTreeMap<String, &Node>) -> String {
    let mut out = String::from("## 関係\n");
    let mut count = 0;
    for edge in &shown.edges {
        if !LINEAGE.contains(&edge.name.as_str()) {
            continue;
        }
        let _ = writeln!(
            out,
            "- {} {}{}",
            edge.name,
            other(edge, targets),
            mark(edge)
        );
        count += 1;
    }
    if count == 0 {
        out.push_str("- 無し\n");
    }
    out.push('\n');
    out
}

fn rest(shown: &Shown, targets: &BTreeMap<String, &Node>) -> String {
    let mut out = shown.verbatim();
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
        let _ = writeln!(
            out,
            "  {} {}{}",
            edge.name,
            other(edge, targets),
            mark(edge)
        );
    }
    if !shown.body.is_empty() {
        let _ = writeln!(out, "{}", shown.body);
    }
    records(&shown.data, &mut out);
    out.push('\n');
    out
}

/// The other side of an edge as `ID (alias) title`, or the bare ID.
fn other(edge: &EdgeLine, targets: &BTreeMap<String, &Node>) -> String {
    targets
        .get(&edge.to)
        .map(|node| reference(node))
        .unwrap_or_else(|| edge.to.clone())
}

fn mark(edge: &EdgeLine) -> String {
    edge.mark
        .as_deref()
        .map(|mark| format!("（mark: {mark}）"))
        .unwrap_or_default()
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
        Some(Closure::Fact) => format!("閉じ方: 事実（{evidence}）"),
        Some(Closure::Decision) => format!("閉じ方: 決定（{evidence}）"),
        Some(Closure::NonDecision) => format!("閉じ方: 決定を伴わない（{evidence}）"),
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
