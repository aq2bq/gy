//! The judgement, undecided, and attention sections: D-85 questions 2 and 6,
//! and the handover diagnostics. Every id is printed with its title.
use super::super::handover::handover;
use super::{in_scope, label};
use crate::model::{Node, NodeData, RequirementState};
use crate::ops::repository::{Repository, Result, Store};
use std::collections::BTreeMap;
use std::fmt::Write;

/// What the master is waiting to decide: the questions with `master` as
/// decider, and the requirements filed but not approved yet.
pub(super) fn judgement<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
) -> Result<String> {
    let mut out = String::new();
    let mut count = 0;
    for node in repository.all()? {
        if !in_scope(&node, scope) {
            continue;
        }
        if master_question(&node) {
            question_item(&mut out, &node);
            count += 1;
        } else if filed_requirement(&node) {
            requirement_item(&mut out, &node);
            count += 1;
        }
    }
    if count == 0 {
        out.push_str("いま判断待ちは無し。\n");
    }
    Ok(out)
}

/// The open questions, grouped by who decides.
pub(super) fn undecided<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
) -> Result<String> {
    let mut by_decider: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in repository.all()? {
        if !in_scope(&node, scope) || !open_question(&node) {
            continue;
        }
        let decider = match node.data() {
            NodeData::Question(data) => data.decider.clone().unwrap_or_default(),
            _ => String::new(),
        };
        by_decider
            .entry(decider)
            .or_default()
            .push(format!("{} {}", label(&node), node.title()));
    }
    if by_decider.is_empty() {
        return Ok("未決の論点は無し。\n".to_string());
    }
    let mut out = String::new();
    for (decider, items) in by_decider {
        let _ = writeln!(out, "### {decider}");
        for item in items {
            let _ = writeln!(out, "- {item}");
        }
    }
    Ok(out)
}

/// The handover warnings and errors as counts by kind; the list itself is not
/// shown (D-87: read it with `handover` or `show`).
pub(super) fn attention<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
) -> Result<String> {
    let report = handover(repository, scope)?;
    let mut out = String::new();
    for warning in &report.warnings {
        let _ = writeln!(
            out,
            "- {}: {}",
            warning_label(&warning.label),
            warning.count
        );
    }
    for (kind, count) in error_counts(&report.errors) {
        let _ = writeln!(out, "- {kind}: {count}");
    }
    if report.warnings.is_empty() && report.errors.is_empty() {
        out.push_str("注意は無し。\n");
    }
    Ok(out)
}

/// Integrity errors grouped by kind. The two shapes come from the handover
/// integrity check; anything else is kept in one more bucket.
fn error_counts(errors: &[String]) -> Vec<(&'static str, usize)> {
    let (mut dangling, mut filed, mut other) = (0, 0, 0);
    for error in errors {
        if error.contains(" missing ") {
            dangling += 1;
        } else if error.contains(" is filed as ") {
            filed += 1;
        } else {
            other += 1;
        }
    }
    let mut out = Vec::new();
    for (kind, count) in [
        ("参照先の無い辺", dangling),
        ("filed-as の先が要求でない", filed),
        ("その他の整合エラー", other),
    ] {
        if count > 0 {
            out.push((kind, count));
        }
    }
    out
}

/// The warning kind in the words of the reader; an unknown label is kept.
fn warning_label(label: &str) -> &str {
    match label {
        "in-progress requirements relying on a superseded decision" => {
            "置き換えられた決定に依拠する進行中の要求"
        }
        "in-progress requirements relying on an unrecorded decision scope" => {
            "成立範囲が未記録の決定に依拠する進行中の要求"
        }
        "criteria with an empty body" => "本文が空の受け入れ条件",
        other => other,
    }
}

fn question_item(out: &mut String, node: &Node) {
    let _ = writeln!(out, "- 論点: {} {}", label(node), node.title());
    if let NodeData::Question(data) = node.data() {
        for option in &data.options {
            let _ = writeln!(out, "  - 選択肢: {option}");
        }
    }
    if let Some(advice) = section(node.body(), "## 推奨") {
        let _ = writeln!(out, "  - 推奨: {}", advice.replace('\n', " "));
    }
}

fn requirement_item(out: &mut String, node: &Node) {
    let _ = writeln!(out, "- 要求: {} {}", label(node), node.title());
    for (name, value) in [
        ("next_evidence", node.free("next_evidence")),
        ("responsible", node.free("responsible")),
    ] {
        if let Some(value) = value {
            let _ = writeln!(out, "  - {name}: {value}");
        }
    }
}

/// The text under a `## ` heading, without the heading itself.
fn section(body: &str, heading: &str) -> Option<String> {
    let start = body.find(heading)? + heading.len();
    let rest = &body[start..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    let text = rest[..end].trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn open_question(node: &Node) -> bool {
    matches!(node.data(), NodeData::Question(data) if data.closure.is_none())
}

fn master_question(node: &Node) -> bool {
    matches!(
        node.data(),
        NodeData::Question(data)
            if data.closure.is_none() && data.decider.as_deref() == Some("master")
    )
}

fn filed_requirement(node: &Node) -> bool {
    matches!(node.data(), NodeData::Requirement(data) if data.state == RequirementState::Filed)
}
