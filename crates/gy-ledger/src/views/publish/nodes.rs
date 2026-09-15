//! The node files: one file per node, named for its kind and ID (d-7c64), in
//! one fixed shape (n-ef16). The body is carried as it is; only the framing is
//! built here.
use super::super::show::{EdgeLine, Shown, show};
use super::{FileEntry, path, reference};
use crate::model::{
    Closed, ClosedBy, Closure, Criterion, Node, NodeData, NodeKind, Question, Requirement,
};
use crate::ops::repository::{Repository, Result, Store};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

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

/// One node file: a heading, the metadata, then the sections (n-ef16).
fn text(shown: &Shown, targets: &BTreeMap<String, &Node>) -> String {
    let mut out = format!("# {}\n\n", summary(shown));
    for line in metadata(shown) {
        let _ = writeln!(out, "- {line}");
    }
    out.push('\n');
    section(&mut out, "関係", &bullets(&relations(shown, targets)));
    if let NodeData::Decision(data) = &shown.data {
        let scope = if data.scope.is_unrecorded() {
            "（未記録）"
        } else {
            data.scope.text()
        };
        section(&mut out, "成立範囲", scope);
    }
    section(&mut out, "本文", shown.body.trim_end());
    if let Some((label, lines)) = records(shown) {
        let lines: Vec<String> = lines.iter().map(|line| format!("- {line}")).collect();
        section(&mut out, label, &bullets(&lines));
    }
    section(&mut out, "自由属性", &bullets(&attributes(shown)));
    out
}

/// `ID (alias) title`, the form every reference uses (d-edb0).
fn summary(shown: &Shown) -> String {
    match shown.aliases.first() {
        Some(alias) => format!("{} ({}) {}", shown.id, alias, shown.title),
        None => format!("{} {}", shown.id, shown.title),
    }
}

fn metadata(shown: &Shown) -> Vec<String> {
    let mut out = vec![format!("種類: {}", shown.kind.name())];
    if let Some(scope) = &shown.scope {
        out.push(format!("scope: {scope}"));
    }
    if let Some(created) = &shown.created {
        out.push(format!("created: {created}"));
    }
    if let Some(state) = state(shown) {
        let reference = match (&shown.data, &shown.reference) {
            (NodeData::Requirement(_), Some(reference)) => format!("（ref: {reference}）"),
            _ => String::new(),
        };
        out.push(format!("状態: {state}{reference}"));
    }
    if !shown.aliases.is_empty() {
        out.push(format!("別名: {}", shown.aliases.join(", ")));
    }
    out
}

/// The node's state; a decision has none.
fn state(shown: &Shown) -> Option<String> {
    match &shown.data {
        NodeData::Need(_) => shown.need.map(str::to_string),
        NodeData::Question(data) => Some(open_closed(data.closure.is_some())),
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

fn open_closed(closed: bool) -> String {
    if closed { "closed" } else { "open" }.to_string()
}

/// Every edge of the node, both directions, once, ordered by name then ID.
fn relations(shown: &Shown, targets: &BTreeMap<String, &Node>) -> Vec<String> {
    let mut edges: Vec<(&EdgeLine, String)> = shown
        .edges
        .iter()
        .map(|edge| {
            (
                edge,
                format!("- {} {}{}", edge.name, other(edge, targets), mark(edge)),
            )
        })
        .collect();
    edges.sort_by(|a, b| (&a.0.name, &a.0.to).cmp(&(&b.0.name, &b.0.to)));
    if edges.is_empty() {
        vec!["- 無し".to_string()]
    } else {
        edges.into_iter().map(|(_, line)| line).collect()
    }
}

fn attributes(shown: &Shown) -> Vec<String> {
    if shown.attributes.is_empty() {
        return vec!["- 無し".to_string()];
    }
    shown
        .attributes
        .iter()
        .map(|(name, value)| format!("- {name}: {value}"))
        .collect()
}

/// The kind's records section: requirements, criteria, and closures.
fn records(shown: &Shown) -> Option<(&'static str, Vec<String>)> {
    match &shown.data {
        NodeData::Requirement(data) => {
            let lines = requirement_records(data);
            (!lines.is_empty()).then_some(("記録", lines))
        }
        NodeData::Criterion(data) => Some(("充足", vec![satisfaction(data)])),
        NodeData::Question(data) => Some(("閉じ方", vec![closure(data)])),
        NodeData::Need(data) => data
            .closed
            .as_ref()
            .map(|closed| ("閉じ方", vec![closed_line(closed)])),
        NodeData::Decision(_) => None,
    }
}

fn requirement_records(data: &Requirement) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(approval) = &data.approval {
        out.push(format!(
            "承認: design={}, heard_by={}, evidence={}, at={}",
            approval.design, approval.heard_by, approval.evidence, approval.at
        ));
    }
    for revision in &data.revisions {
        out.push(format!(
            "改訂: {}（出典: {}、{}）",
            revision.reason, revision.source, revision.at
        ));
    }
    if let Some(completion) = &data.completion {
        out.push(format!(
            "完了: {}（{}）",
            completion.evidence, completion.at
        ));
    }
    if let Some(cancellation) = &data.cancellation {
        out.push(format!(
            "中止: {}（出典: {}、{}）",
            cancellation.reason, cancellation.source, cancellation.at
        ));
    }
    out
}

fn satisfaction(data: &Criterion) -> String {
    if !data.satisfied {
        return "unsatisfied".to_string();
    }
    let mut line = "satisfied".to_string();
    if let Some(evidence) = &data.evidence {
        line.push_str(&format!("（{evidence}）"));
    }
    if let Some(at) = &data.satisfied_at {
        line.push_str(&format!(" {at}"));
    }
    line
}

fn closure(data: &Question) -> String {
    let evidence = data.evidence.clone().unwrap_or_default();
    match data.closure {
        Some(Closure::Fact) => format!("事実で閉じた（{evidence}）"),
        Some(Closure::Decision) => format!("決定で閉じた（{evidence}）"),
        Some(Closure::NonDecision) => format!("決定を伴わず閉じた（{evidence}）"),
        None => "開いている".to_string(),
    }
}

fn closed_line(closed: &Closed) -> String {
    let by = match closed.by {
        ClosedBy::Fact => "事実",
        ClosedBy::External => "外部",
    };
    format!("{by}で閉じた（{}）", closed.evidence)
}

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

fn bullets(lines: &[String]) -> String {
    lines.join("\n")
}

/// One `## ` section, with a blank line before the heading and after the body.
fn section(out: &mut String, heading: &str, body: &str) {
    let _ = write!(out, "## {heading}\n\n");
    let body = body.trim_end();
    if !body.is_empty() {
        let _ = writeln!(out, "{body}");
        out.push('\n');
    }
}
