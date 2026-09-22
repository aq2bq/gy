//! A vertex's page (n-b6c9, d-50d1): the sections a reader follows in place,
//! the criterion / requirement / question each expanded where it is reached.
use super::super::show::Shown;
use super::inline;
use super::wiki::Scope;
use crate::model::{ClosedBy, NodeData};

pub(super) fn need_page(scope: &Scope, node: &Shown) -> String {
    let here = format!("{}.md", node.id);
    let mut out = scope.page_head(node);
    out += &comes_from(scope, node, &here);
    section(&mut out, "Why", &scope.marked(node, &node.body));
    let targets = edge_ids(scope, &node.id, "targets");
    out += &inline::section(scope, "What must hold", &targets, &here);
    let filed = edge_ids(scope, &node.id, "filed-as");
    out += &inline::section(scope, "What was built", &filed, &here);
    let raised: Vec<String> = filed
        .iter()
        .flat_map(|requirement| edge_ids(scope, requirement, "raised"))
        .collect();
    out += &inline::section(scope, "What it raised", &raised, &here);
    let waits = edge_ids(scope, &node.id, "waits-on");
    out += &inline::section(scope, "What it waits on", &waits, &here);
    out += &ended(node);
    out
}

pub(super) fn decision_page(scope: &Scope, node: &Shown) -> String {
    let here = format!("{}.md", node.id);
    let mut out = scope.page_head(node);
    section(
        &mut out,
        "Where it holds",
        &scope.marked(node, scope_text(node)),
    );
    let lineage = lineage(scope, node, &here);
    if !lineage.is_empty() {
        out += &format!("## Its lineage\n\n{lineage}\n");
    }
    section(
        &mut out,
        "How it came about",
        &scope.marked(node, &node.body),
    );
    let closed = edge_ids(scope, &node.id, "closed-by");
    out += &inline::section(scope, "The questions it closed", &closed, &here);
    out += &came_out(scope, node, &here);
    out
}

/// The ids a node points at with one relation.
fn edge_ids(scope: &Scope, id: &str, name: &str) -> Vec<String> {
    scope
        .edges(id, name)
        .iter()
        .map(|edge| edge.to.clone())
        .collect()
}

fn comes_from(scope: &Scope, node: &Shown, here: &str) -> String {
    let spawned = scope.edges(&node.id, "spawned-by");
    let depends = scope.edges(&node.id, "depends-on");
    if spawned.is_empty() && depends.is_empty() {
        return String::new();
    }
    let mut out = String::from("## What it comes from\n\n");
    for edge in spawned {
        out += &format!("- Spawned by the decision {}\n", scope.link(&edge.to, here));
    }
    for edge in depends {
        out += &format!("- Waits for {}\n", scope.link(&edge.to, here));
    }
    out.push('\n');
    out
}

fn ended(node: &Shown) -> String {
    let NodeData::Need(data) = &node.data else {
        return String::new();
    };
    let Some(closed) = &data.closed else {
        return String::new();
    };
    let by = match closed.by {
        ClosedBy::Fact => "fact",
        ClosedBy::External => "external",
    };
    format!("## How it ended\n\nClosed by {by}: {}\n", closed.evidence)
}

/// A decision's applicability notes, empty for one the migration left unrecorded.
fn scope_text(node: &Shown) -> &str {
    match &node.data {
        NodeData::Decision(data) if !data.scope.is_unrecorded() => data.scope.text(),
        _ => "",
    }
}

/// The decisions a decision narrows, widens, replaces or completes, and the
/// later decisions that did so to it, each with the passage that stops applying.
fn lineage(scope: &Scope, node: &Shown, here: &str) -> String {
    let mut out = String::new();
    for (name, phrase) in [
        ("narrows", "Narrows"),
        ("widens", "Widens"),
        ("supersedes", "Replaces"),
        ("completes", "Completes"),
        ("narrowed-by", "Narrowed by"),
        ("widened-by", "Widened by"),
        ("superseded-by", "Replaced by"),
        ("completed-by", "Completed by"),
    ] {
        for edge in scope.edges(&node.id, name) {
            out += &format!("- {phrase} {}\n", scope.link(&edge.to, here));
            if let Some(mark) = &edge.mark {
                out += &format!("  - The passage that stops applying: \"{mark}\"\n");
            }
        }
    }
    out
}

fn came_out(scope: &Scope, node: &Shown, here: &str) -> String {
    let spawns = scope.edges(&node.id, "spawns");
    let relied = scope.edges(&node.id, "relied-on-by");
    if spawns.is_empty() && relied.is_empty() {
        return String::new();
    }
    let mut out = String::from("## What came out of it\n\n");
    for edge in spawns {
        out += &format!("- The need {}\n", scope.link(&edge.to, here));
    }
    for edge in relied {
        out += &format!("- Relied on by {}\n", scope.link(&edge.to, here));
    }
    out.push('\n');
    out
}

/// One `## ` section, with a blank line before the heading and after the body.
fn section(out: &mut String, heading: &str, body: &str) {
    let body = body.trim_end();
    if body.is_empty() {
        return;
    }
    out.push_str(&format!("## {heading}\n\n{body}\n\n"));
}
