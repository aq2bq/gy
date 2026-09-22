//! show: one node, or several, in the verbatim a kind calls for, or in full
//! (proposal-v3 4). A requirement always shows its id with its ref beside it.
use super::derive::need_state;
use super::open_or_closed;
use super::retraction::{Retraction, retracted, retractions_by_node, scope_marked};
use crate::model::{Criterion, Edge, Node, NodeData, NodeKind};
use crate::ops::repository::{Error, Repository, Result, Snapshot, Store};
use crate::ops::{advice, local_time};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;

/// One edge as a view prints it: the name from this node's side, the other id,
/// and the mark a narrows / supersedes carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EdgeLine {
    pub name: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<String>,
}
impl EdgeLine {
    fn of(edge: &Edge) -> Self {
        Self {
            name: edge.name().to_string(),
            to: edge.to.to_string(),
            mark: edge.mark.clone(),
        }
    }
    fn line(&self) -> String {
        match &self.mark {
            Some(mark) => format!("  {} {} ({})", self.name, self.to, mark),
            None => format!("  {} {}", self.name, self.to),
        }
    }
}

/// A node as a view projects it. `data` stays typed, so `--json` is the model
/// and `Display` is the human rendering.
#[derive(Debug, Clone, Serialize)]
pub struct Shown {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub kind: NodeKind,
    pub title: String,
    pub body: String,
    /// The body with its narrowed passages marked; `None` when nothing of it
    /// loses effect. The raw `body` stays the record (n-0c6f).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_marked: Option<String>,
    /// The applicability conditions with their narrowed passages marked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_scope_marked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub data: NodeData,
    /// What later decisions retract of this one (n-0c6f, d-bde9).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<Retraction>,
    /// A need's derived state (open / closed / done), from the graph (n-35cf).
    #[serde(skip_serializing)]
    pub need: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<EdgeLine>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}
impl Shown {
    fn new(
        node: &Node,
        incoming: &[Edge],
        cancellation: Option<Retraction>,
        all: &[Node],
        full: bool,
    ) -> Self {
        let reference = match node.data() {
            NodeData::Requirement(data) => data.reference.as_ref().map(|ref_| ref_.0.clone()),
            _ => None,
        };
        let body = projected_body(node, full, cancellation.as_ref());
        let body_marked = cancellation
            .as_ref()
            .and_then(|retraction| retracted(&body, retraction));
        let decision_scope_marked = scope_marked(node, cancellation.as_ref());
        Self {
            id: node.id().to_string(),
            reference,
            kind: node.kind(),
            title: node.title().to_string(),
            body,
            body_marked,
            decision_scope_marked,
            scope: full.then(|| node.scope().to_string()),
            created: full.then(|| node.created().to_string()),
            aliases: projected_aliases(node, full),
            data: node.data().clone(),
            cancellation,
            need: match node.data() {
                NodeData::Need(_) => Some(need_state(node, all).name()),
                _ => None,
            },
            edges: projected_edges(node, incoming, full),
            missing: advice::missing(node, all),
            attributes: projected_attributes(node, full),
        }
    }
    fn heading(&self) -> String {
        match &self.reference {
            Some(reference) => format!("{} ({reference}) {}", self.id, self.title),
            None => format!("{} {}", self.id, self.title),
        }
    }
    /// The kind-specific verbatim of the default view, without the heading.
    pub fn verbatim(&self) -> String {
        let mut out = String::new();
        match &self.data {
            NodeData::Need(_) => {
                let _ = writeln!(out, "state: {}", self.need.unwrap_or("open"));
            }
            NodeData::Question(data) => {
                let _ = writeln!(out, "state: {}", open_or_closed(data.closure.is_some()));
                if let Some(decider) = &data.decider {
                    let _ = writeln!(out, "decider: {decider}");
                }
                if !data.options.is_empty() {
                    let _ = writeln!(out, "options: {}", data.options.join(", "));
                }
            }
            NodeData::Decision(data) => {
                let scope = if data.scope.is_unrecorded() {
                    "(unrecorded)".to_string()
                } else {
                    self.decision_scope_marked
                        .clone()
                        .unwrap_or_else(|| data.scope.text().to_string())
                };
                let _ = writeln!(out, "decision_scope: {scope}");
            }
            NodeData::Requirement(data) => {
                let _ = writeln!(out, "state: {}", data.state.name());
            }
            NodeData::Criterion(data) => {
                let _ = writeln!(out, "{}", criterion_line(data));
            }
        }
        out
    }
}
impl fmt::Display for Shown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.heading())?;
        if let Some(retraction) = &self.cancellation {
            for id in &retraction.superseded_by {
                writeln!(f, "superseded-by {id}")?;
            }
            for narrow in &retraction.narrowed {
                writeln!(f, "narrowed-by {} ({})", narrow.by, narrow.mark)?;
            }
        }
        f.write_str(&self.verbatim())?;
        for (name, value) in &self.attributes {
            writeln!(f, "{name}: {value}")?;
        }
        for edge in &self.edges {
            writeln!(f, "{}", edge.line())?;
        }
        if let Some(scope) = &self.scope {
            writeln!(f, "scope: {scope}")?;
        }
        if let Some(created) = &self.created {
            writeln!(f, "created: {}", local_time(created))?;
        }
        if !self.aliases.is_empty() {
            writeln!(f, "aliases: {}", self.aliases.join(", "))?;
        }
        if let Some(marked) = &self.body_marked {
            writeln!(f, "{marked}")?;
        } else if !self.body.is_empty() {
            writeln!(f, "{}", self.body)?;
        }
        if !self.missing.is_empty() {
            writeln!(f, "missing: {}", self.missing.join(", "))?;
        }
        Ok(())
    }
}

/// Project the named texts in order; several matches is an error (proposal-v3 11).
pub fn show<S: Store>(repo: &Repository<S>, texts: &[String], full: bool) -> Result<Vec<Shown>> {
    show_with(&repo.snapshot()?, texts, full)
}

/// Every node, from one store pass (n-2e03); for publish, which reads them all.
pub(crate) fn show_all<S: Store>(repo: &Repository<S>, full: bool) -> Result<Vec<Shown>> {
    let snapshot = repo.snapshot()?;
    let texts: Vec<String> = snapshot
        .nodes()
        .iter()
        .map(|node| node.id().to_string())
        .collect();
    show_with(&snapshot, &texts, full)
}
fn show_with(snapshot: &Snapshot, texts: &[String], full: bool) -> Result<Vec<Shown>> {
    let all = snapshot.nodes();
    let retractions = retractions_by_node(all);
    let mut out = Vec::new();
    for text in texts {
        let id = snapshot.resolve(text)?;
        let node = snapshot
            .get(&id)
            .ok_or_else(|| Error::invalid(format!("{id} does not exist")))?;
        let incoming = snapshot.incoming(&id);
        let cancellation = retractions.get(&id.to_string()).cloned();
        out.push(Shown::new(node, incoming, cancellation, all, full));
    }
    Ok(out)
}

fn criterion_line(data: &Criterion) -> String {
    let mut line = format!("satisfied: {}", data.satisfied);
    if let Some(evidence) = &data.evidence {
        line.push_str(&format!(" ({evidence})"));
    }
    if let Some(at) = &data.satisfied_at {
        line.push_str(&format!(" at {}", local_time(at)));
    }
    line
}

fn projected_body(node: &Node, full: bool, cancellation: Option<&Retraction>) -> String {
    let NodeData::Decision(data) = node.data() else {
        return node.body().to_string();
    };
    if full {
        return node.body().to_string();
    }
    let section = decision_section(node.body());
    if let Some(retraction) = cancellation {
        let scope = retracted(data.scope.text(), retraction).is_some();
        if !retraction.narrowed.is_empty() && retracted(&section, retraction).is_none() && !scope {
            return node.body().to_string();
        }
    }
    section
}

/// The `## Decision` section, up to the next `## ` heading; without one, the
/// whole body (proposal-v3 4).
fn decision_section(body: &str) -> String {
    let Some(start) = body.find("## Decision") else {
        return body.to_string();
    };
    let rest = &body[start..];
    let end = rest[1..].find("\n## ").map_or(rest.len(), |at| at + 1);
    rest[..end].trim_end().to_string()
}

fn projected_aliases(node: &Node, full: bool) -> Vec<String> {
    if full {
        node.aliases().iter().map(|alias| alias.0.clone()).collect()
    } else {
        Vec::new()
    }
}

fn projected_edges(node: &Node, incoming: &[Edge], full: bool) -> Vec<EdgeLine> {
    if !full && node.kind() != NodeKind::Requirement {
        return Vec::new();
    }
    let mut edges: Vec<EdgeLine> = node.links().iter().map(EdgeLine::of).collect();
    if full {
        edges.extend(incoming.iter().map(EdgeLine::of));
    }
    edges
}

/// Requirements show next_evidence and responsible; full shows every free
/// attribute.
fn projected_attributes(node: &Node, full: bool) -> BTreeMap<String, String> {
    if full {
        return node.free_attributes().clone();
    }
    let mut out = BTreeMap::new();
    if node.kind() == NodeKind::Requirement {
        for name in ["next_evidence", "responsible"] {
            if let Some(value) = node.free(name) {
                out.insert(name.to_string(), value.clone());
            }
        }
    }
    out
}
