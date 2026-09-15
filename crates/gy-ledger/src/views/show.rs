//! show: one node, or several, in the verbatim a kind calls for, or in full
//! (proposal-v3 4). A requirement always shows its id with its ref beside it.
use super::open_or_closed;
use crate::model::{Criterion, Edge, Node, NodeData, NodeKind};
use crate::ops::repository::{Error, Repository, Result, Store};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub data: NodeData,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<EdgeLine>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}
impl Shown {
    fn new(node: &Node, incoming: &[Edge], full: bool) -> Self {
        let reference = match node.data() {
            NodeData::Requirement(data) => data.reference.as_ref().map(|ref_| ref_.0.clone()),
            _ => None,
        };
        Self {
            id: node.id().to_string(),
            reference,
            kind: node.kind(),
            title: node.title().to_string(),
            body: projected_body(node, full),
            scope: full.then(|| node.scope().to_string()),
            created: full.then(|| node.created().to_string()),
            aliases: projected_aliases(node, full),
            data: node.data().clone(),
            edges: projected_edges(node, incoming, full),
            attributes: projected_attributes(node, full),
        }
    }
    fn heading(&self) -> String {
        match &self.reference {
            Some(reference) => format!("{} ({reference}) {}", self.id, self.title),
            None => format!("{} {}", self.id, self.title),
        }
    }
    fn show_data(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            NodeData::Need(data) => writeln!(f, "state: {}", open_or_closed(data.closed.is_some())),
            NodeData::Question(data) => {
                writeln!(f, "state: {}", open_or_closed(data.closure.is_some()))?;
                if let Some(decider) = &data.decider {
                    writeln!(f, "decider: {decider}")?;
                }
                if !data.options.is_empty() {
                    writeln!(f, "options: {}", data.options.join(", "))?;
                }
                Ok(())
            }
            NodeData::Decision(data) => {
                let scope = if data.scope.is_unrecorded() {
                    "(unrecorded)".to_string()
                } else {
                    data.scope.text().to_string()
                };
                writeln!(f, "decision_scope: {scope}")
            }
            NodeData::Requirement(data) => writeln!(f, "state: {}", data.state.name()),
            NodeData::Criterion(data) => writeln!(f, "{}", criterion_line(data)),
        }
    }
}
impl fmt::Display for Shown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.heading())?;
        self.show_data(f)?;
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
            writeln!(f, "created: {created}")?;
        }
        if !self.aliases.is_empty() {
            writeln!(f, "aliases: {}", self.aliases.join(", "))?;
        }
        if !self.body.is_empty() {
            writeln!(f, "{}", self.body)?;
        }
        Ok(())
    }
}

/// Project the named texts in order. Resolving any one of them is an error
/// that names the candidates when several refs match (proposal-v3 11).
pub fn show<S: Store>(repo: &Repository<S>, texts: &[String], full: bool) -> Result<Vec<Shown>> {
    let mut out = Vec::new();
    for text in texts {
        let id = repo.resolve(text)?;
        let node = repo
            .get(&id)?
            .ok_or_else(|| Error::invalid(format!("{id} does not exist")))?;
        let incoming = if full {
            repo.incoming(&id)?
        } else {
            Vec::new()
        };
        out.push(Shown::new(&node, &incoming, full));
    }
    Ok(out)
}

fn criterion_line(data: &Criterion) -> String {
    let mut line = format!("satisfied: {}", data.satisfied);
    if let Some(evidence) = &data.evidence {
        line.push_str(&format!(" ({evidence})"));
    }
    if let Some(at) = &data.satisfied_at {
        line.push_str(&format!(" at {at}"));
    }
    line
}

fn projected_body(node: &Node, full: bool) -> String {
    match node.data() {
        NodeData::Decision(_) if !full => decision_section(node.body()),
        _ => node.body().to_string(),
    }
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
