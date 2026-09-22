//! The wiki's entry page (n-a391, d-82d6): one README per scope, the way in.
//! The counts and the record's seq, the open questions, the work being built,
//! the newest and the full lists. The old index's reading, history and
//! diagnostics are gone (d-d82b): a reader follows links, not a legend.
use super::super::show::Shown;
use super::wiki::Scope;
use crate::model::{NodeData, NodeKind, RequirementState};

/// A link from the entry: the page that shows the node, or `loose.md` for one
/// no vertex expands.
fn entry_link(scope: &Scope, id: &str) -> String {
    let title = scope
        .node(id)
        .map_or_else(|| id.to_string(), |node| node.title.clone());
    let page = scope.page_of(id).unwrap_or_else(|| "loose.md".to_string());
    format!(
        "[`{id}` {title}]({page}{})",
        super::wiki::anchor(scope.node(id))
    )
}

pub(super) fn readme(scope: &Scope, seq: u64, loose: bool) -> String {
    let nodes = in_scope(scope);
    let mut out = format!("# {}\n\n", scope.name());
    out += &format!(
        "{} nodes, read as {} pages: one for every need and every decision, with the \
         questions, requirements and criteria that hang off them shown in place.\n\n",
        nodes.len(),
        scope.vertices().len()
    );
    out += &format!("seq: {seq}\n\n");
    out += &section("Undecided", &questions(scope, &nodes));
    out += &section("Being built", &building(scope, &nodes));
    out += &latest(scope, &nodes);
    out += &section(
        &format!("Decisions ({})", count(&nodes, NodeKind::Decision)),
        &vertices(scope, &nodes, NodeKind::Decision),
    );
    out += &section(
        &format!("Needs ({})", count(&nodes, NodeKind::Need)),
        &vertices(scope, &nodes, NodeKind::Need),
    );
    if loose {
        out += "- [On their own](loose.md) — what no need and no decision reaches.\n";
    }
    out
}

fn in_scope<'a>(scope: &'a Scope) -> Vec<&'a Shown> {
    scope
        .all_ids()
        .iter()
        .filter_map(|id| scope.node(id))
        .filter(|node| node.scope.as_deref() == Some(scope.name()))
        .collect()
}

/// One `## ` section from its lines, as a list, with a word where there are
/// none. The lines carry no marker of their own; GitHub needs the `- ` on each
/// line or a newline-joined run renders as one paragraph.
fn section(title: &str, lines: &[String]) -> String {
    if lines.is_empty() {
        return format!("## {title}\n\n- Nothing.\n\n");
    }
    let mut out = format!("## {title}\n\n");
    for line in lines {
        out += &format!("- {line}\n");
    }
    out.push('\n');
    out
}

fn questions(scope: &Scope, nodes: &[&Shown]) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| matches!(&node.data, NodeData::Question(data) if data.closure.is_none()))
        .map(|node| entry_link(scope, &node.id))
        .collect()
}

fn building(scope: &Scope, nodes: &[&Shown]) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| {
            matches!(&node.data, NodeData::Requirement(data)
                if matches!(data.state, RequirementState::Filed | RequirementState::Approved))
        })
        .map(|node| entry_link(scope, &node.id))
        .collect()
}

fn latest(scope: &Scope, nodes: &[&Shown]) -> String {
    let lines: Vec<String> = newest(scope, nodes).into_iter().take(10).collect();
    section("Latest", &lines)
}

fn vertices(scope: &Scope, nodes: &[&Shown], kind: NodeKind) -> Vec<String> {
    let mut kept: Vec<&&Shown> = nodes.iter().filter(|node| node.kind == kind).collect();
    kept.sort_by(|a, b| b.created.cmp(&a.created));
    kept.iter()
        .map(|node| entry_link(scope, &node.id))
        .collect()
}

/// The vertices' links, newest first, each with its date.
fn newest(scope: &Scope, nodes: &[&Shown]) -> Vec<String> {
    let mut kept: Vec<&&Shown> = nodes
        .iter()
        .filter(|node| super::is_vertex(node.kind))
        .collect();
    kept.sort_by(|a, b| b.created.cmp(&a.created));
    kept.iter()
        .map(|node| {
            let date = node
                .created
                .as_deref()
                .and_then(|created| created.get(..10))
                .unwrap_or("");
            format!("{date} — {}", entry_link(scope, &node.id))
        })
        .collect()
}

fn count(nodes: &[&Shown], kind: NodeKind) -> usize {
    nodes.iter().filter(|node| node.kind == kind).count()
}
