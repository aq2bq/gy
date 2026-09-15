//! The node description: one named node with its title, state, verbatim, and
//! its relationships in human words (D-85 questions 1, 3, 4; D-87 names the
//! nodes).
use super::super::show::show;
use super::label;
use crate::model::{Node, NodeData, NodeId, NodeKind, Relation, Requirement};
use crate::ops::repository::{Error, Repository, Result, Store};
use std::fmt::Write;

/// The description of each named node, and nothing else.
pub(super) fn describe<S: Store>(repository: &Repository<S>, ids: &[NodeId]) -> Result<String> {
    let mut out = String::new();
    for id in ids {
        let node = repository
            .get(id)?
            .ok_or_else(|| Error::invalid(format!("{id} does not exist")))?;
        out.push_str(&section(repository, &node)?);
    }
    Ok(out)
}

fn section<S: Store>(repository: &Repository<S>, node: &Node) -> Result<String> {
    let mut out = format!("### {} {}\n", label(node), node.title());
    if let Some(shown) = show(repository, &[node.id().to_string()], false)?.first() {
        out.push_str(&shown.verbatim());
    }
    out.push_str(&relationships(repository, node)?);
    if let NodeData::Requirement(data) = node.data() {
        out.push_str(&provenance(repository, node, data)?);
    }
    out.push('\n');
    Ok(out)
}

/// Every edge of this node, forward and reverse, as a sentence (question 3 is
/// the narrows / supersedes part of this with its mark).
fn relationships<S: Store>(repository: &Repository<S>, node: &Node) -> Result<String> {
    let mut out = String::from("関係:\n");
    let mut count = 0;
    for edge in node.links() {
        if let Some(other) = repository.get(&edge.to)? {
            let _ = writeln!(
                out,
                "- {}",
                phrase(node.kind(), &other, edge.name(), edge.mark.as_deref())
            );
            count += 1;
        }
    }
    for edge in repository.incoming(node.id())? {
        if let Some(other) = repository.get(&edge.to)? {
            let _ = writeln!(
                out,
                "- {}",
                phrase(node.kind(), &other, edge.name(), edge.mark.as_deref())
            );
            count += 1;
        }
    }
    if count == 0 {
        out.push_str("- 無し\n");
    }
    Ok(out)
}

fn phrase(kind: NodeKind, other: &Node, name: &str, mark: Option<&str>) -> String {
    let (particle, verb) = words(name);
    let base = format!(
        "この{}は {} {} {particle}{verb}",
        kind_word(kind),
        label(other),
        other.title()
    );
    match mark {
        Some(mark) => format!("{base}（mark: {mark}）"),
        None => base,
    }
}

/// The requirements provenance: its needs, the decisions it relies on, and its
/// state records (question 4).
fn provenance<S: Store>(
    repository: &Repository<S>,
    node: &Node,
    data: &Requirement,
) -> Result<String> {
    let mut out = String::from("来歴:\n");
    origins(repository, node, &mut out)?;
    records(data, &mut out);
    Ok(out)
}

/// The needs this requirement came from and the decisions it relies on.
fn origins<S: Store>(repository: &Repository<S>, node: &Node, out: &mut String) -> Result<()> {
    for edge in repository.incoming(node.id())? {
        if edge.label == Relation::FiledAs {
            if let Some(need) = repository.get(&edge.to)? {
                let _ = writeln!(out, "- ニーズ: {} {}", label(&need), need.title());
            }
        }
    }
    for edge in node.links() {
        if edge.label == Relation::ReliesOn {
            if let Some(decision) = repository.get(&edge.to)? {
                let _ = writeln!(
                    out,
                    "- 依拠する決定: {} {}{}",
                    label(&decision),
                    decision.title(),
                    unrecorded(&decision)
                );
            }
        }
    }
    Ok(())
}

fn records(data: &Requirement, out: &mut String) {
    let _ = writeln!(out, "- 状態: {}", data.state.name());
    if let Some(approval) = &data.approval {
        let _ = writeln!(
            out,
            "- 承認: design={}, heard_by={}, evidence={}, at={}",
            approval.design, approval.heard_by, approval.evidence, approval.at
        );
    }
    for revision in &data.revisions {
        let _ = writeln!(
            out,
            "- 改訂: {}（出典: {}、{}）",
            revision.reason, revision.source, revision.at
        );
    }
    if let Some(completion) = &data.completion {
        let _ = writeln!(out, "- 完了: {}（{}）", completion.evidence, completion.at);
    }
    if let Some(cancellation) = &data.cancellation {
        let _ = writeln!(
            out,
            "- 中止: {}（出典: {}、{}）",
            cancellation.reason, cancellation.source, cancellation.at
        );
    }
}

fn unrecorded(node: &Node) -> &'static str {
    match node.data() {
        NodeData::Decision(data) if data.scope.is_unrecorded() => "（成立範囲: 未記録）",
        _ => "",
    }
}

fn kind_word(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Need => "ニーズ",
        NodeKind::Question => "論点",
        NodeKind::Decision => "決定",
        NodeKind::Requirement => "要求",
        NodeKind::Criterion => "受け入れ条件",
    }
}

/// The particle and verb a relation phrase uses, from this node's side.
fn words(name: &str) -> (&'static str, &'static str) {
    match name {
        "closes" => ("を", "閉じる"),
        "closed-by" => ("に", "閉じられている"),
        "narrows" => ("を", "狭める"),
        "narrowed-by" => ("に", "狭められている"),
        "widens" => ("を", "広げる"),
        "widened-by" => ("に", "広げられている"),
        "supersedes" => ("を", "置き換える"),
        "superseded-by" => ("に", "置き換えられている"),
        "completes" => ("を", "完成させる"),
        "completed-by" => ("に", "完成させられている"),
        "targets" => ("を", "対象にする"),
        "targeted-by" => ("に", "対象にされている"),
        "spawns" => ("を", "生む"),
        "spawned-by" => ("から", "生まれた"),
        "files" => ("を", "起票元にする"),
        "filed-as" => ("として", "起票した"),
        "depends-on" => ("に", "依存する"),
        "depended-on-by" => ("から", "依存されている"),
        "relies-on" => ("に", "依拠する"),
        "relied-on-by" => ("から", "依拠されている"),
        "raised" => ("を", "提起した"),
        "raised-by" => ("から", "提起されている"),
        "waits-on" => ("を", "待つ"),
        "awaited-by" => ("から", "待たれている"),
        _ => ("を", "関係する"),
    }
}
