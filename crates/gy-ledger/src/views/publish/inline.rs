//! The nodes a vertex's page expands in full (n-b6c9, d-50d1): a criterion, a
//! requirement, a question, each with the record the type carries.
use super::super::show::Shown;
use super::wiki::Scope;
use crate::model::{Closure, Criterion, NodeData, NodeKind, Question, Requirement};

/// One `## ` section of the inlines whose ids are in scope, or nothing when
/// none of them is a node this page shows.
pub(super) fn section(scope: &Scope, title: &str, ids: &[String], here: &str) -> String {
    let mut out = String::new();
    for id in ids {
        let Some(node) = scope.node(id) else {
            continue;
        };
        if node.scope.as_deref() != Some(scope.name()) || !super::is_inline(node.kind) {
            continue;
        }
        if out.is_empty() {
            out += &format!("## {title}\n\n");
        }
        out += &block(scope, node, here);
        out.push('\n');
    }
    out
}

/// One inline node and, when other vertices show it too, who shares it.
pub(super) fn block(scope: &Scope, node: &Shown, here: &str) -> String {
    let mut out = match &node.data {
        NodeData::Question(data) => question(node, data, scope),
        NodeData::Requirement(data) => requirement(node, data, scope, here),
        NodeData::Criterion(data) => criterion(node, data, scope),
        _ => String::new(),
    };
    let shared = scope.shared(&node.id, here);
    if !shared.is_empty() {
        let links: Vec<String> = shared.iter().map(|id| scope.link(id, here)).collect();
        out += &format!("- Shared with: {}\n", links.join(", "));
    }
    out
}

fn heading(node: &Shown) -> String {
    let label = match node.kind {
        NodeKind::Question => "Question",
        NodeKind::Requirement => "Requirement",
        NodeKind::Criterion => "Criterion",
        _ => "Node",
    };
    format!(
        "#### <a id=\"{}\"></a>{label} `{}` — {}\n",
        node.id.replace('-', ""),
        node.id,
        node.title
    )
}

fn question(node: &Shown, data: &Question, scope: &Scope) -> String {
    let mut out = heading(node);
    if let Some(decider) = &data.decider {
        out += &format!("- Decider: {decider}\n");
    }
    for option in &data.options {
        out += &format!("- Option: {option}\n");
    }
    if let Some(closure) = data.closure {
        let way = match closure {
            Closure::Fact => "fact",
            Closure::Decision => "decision",
            Closure::NonDecision => "non-decision",
        };
        let evidence = data.evidence.clone().unwrap_or_default();
        if evidence.is_empty() {
            out += &format!("- Closed by {way}\n");
        } else {
            out += &format!("- Closed by {way}: {evidence}\n");
        }
    }
    body(&mut out, scope.marked(node, &node.body));
    out
}

fn requirement(node: &Shown, data: &Requirement, scope: &Scope, here: &str) -> String {
    let mut out = heading(node);
    out += &format!("- State: **{}**\n", data.state.name());
    if let Some(approval) = &data.approval {
        out += &format!(
            "- Design heard by {}: {}\n",
            approval.heard_by, approval.design
        );
        if !approval.evidence.is_empty() {
            out += &format!("- Approved on: {}\n", approval.evidence);
        }
    }
    if let Some(completion) = &data.completion {
        out += &format!("- Done: {}\n", completion.evidence);
    }
    if let Some(cancellation) = &data.cancellation {
        out += &format!("- Cancelled: {}\n", cancellation.reason);
    }
    for revision in &data.revisions {
        out += &format!("- Revised: {}\n", revision.reason);
    }
    let relies = scope.edges(&node.id, "relies-on");
    if !relies.is_empty() {
        let links: Vec<String> = relies
            .iter()
            .map(|edge| scope.link(&edge.to, here))
            .collect();
        out += &format!("- Relies on: {}\n", links.join(", "));
    }
    body(&mut out, scope.marked(node, &node.body));
    out
}

fn criterion(node: &Shown, data: &Criterion, scope: &Scope) -> String {
    let mut out = heading(node);
    out += if data.satisfied {
        "- Met\n"
    } else {
        "- Not met yet\n"
    };
    if data.satisfied {
        if let Some(evidence) = &data.evidence {
            out += &format!("- Evidence: {evidence}\n");
        }
    }
    let text = scope.marked(node, &node.body);
    if !text.trim().is_empty() {
        out += &format!("\nHow it is measured:\n\n{}\n", text.trim_end());
    }
    out
}

/// A node's body, on its own paragraph when it has one.
fn body(out: &mut String, text: String) {
    if !text.trim().is_empty() {
        out.push_str(&format!("\n{}\n", text.trim_end()));
    }
}
