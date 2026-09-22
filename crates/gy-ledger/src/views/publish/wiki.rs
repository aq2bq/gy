//! The wiki's context (n-b6c9, d-d82b): every node's projection, and, per
//! scope, which page shows which node. The owner of a node is the vertex whose
//! page expands it in full — a need's targets / filed-as / raised / waits-on,
//! or a decision's closed-by. A relation that only refers (a requirement's
//! relies-on, the reverse of spawned-by) does not make a page its owner, so a
//! node left over falls to `loose.md` and never goes missing.
use super::super::retraction::{Narrowed, Retraction};
use super::super::show::{EdgeLine, Shown};
use super::{FileEntry, is_inline, is_vertex};
use crate::model::{NodeData, NodeKind};
use std::collections::{BTreeMap, BTreeSet};

/// Every node as the view projects it, in the repository's order.
pub(super) struct Wiki {
    pub(super) order: Vec<String>,
    nodes: BTreeMap<String, Shown>,
}
impl Wiki {
    pub(super) fn new(shown: Vec<Shown>) -> Self {
        let order = shown.iter().map(|node| node.id.clone()).collect();
        Self {
            order,
            nodes: shown
                .into_iter()
                .map(|node| (node.id.clone(), node))
                .collect(),
        }
    }
    pub(super) fn get(&self, id: &str) -> Option<&Shown> {
        self.nodes.get(id)
    }
    /// The files of one scope: one page per need and decision, then `loose.md`.
    pub(super) fn files(&self, name: &str) -> Vec<FileEntry> {
        let scope = Scope::new(self, name);
        let mut files = Vec::new();
        for vertex in &scope.order {
            let node = self.get(vertex).expect("a vertex is a node");
            let text = match node.kind {
                NodeKind::Need => super::page::need_page(&scope, node),
                NodeKind::Decision => super::page::decision_page(&scope, node),
                _ => continue,
            };
            files.push(FileEntry {
                path: format!("{vertex}.md"),
                text,
            });
        }
        if let Some(text) = super::loose::loose_page(&scope) {
            files.push(FileEntry {
                path: "loose.md".to_string(),
                text,
            });
        }
        files
    }
}

/// One scope: the vertices that have pages, and which of them shows a node.
pub(super) struct Scope<'a> {
    wiki: &'a Wiki,
    name: String,
    order: Vec<String>,
    inlined: BTreeMap<String, Vec<String>>,
    owner: BTreeMap<String, String>,
}
impl<'a> Scope<'a> {
    pub(super) fn new(wiki: &'a Wiki, name: &str) -> Self {
        let order: Vec<String> = wiki
            .order
            .iter()
            .filter(|id| in_scope(wiki, id, name) && is_vertex(wiki.get(id).unwrap().kind))
            .cloned()
            .collect();
        let mut inlined = BTreeMap::new();
        let mut owner = BTreeMap::new();
        for vertex in &order {
            let ids = inlined_by(wiki, wiki.get(vertex).unwrap(), name);
            for id in &ids {
                owner.entry(id.clone()).or_insert_with(|| vertex.clone());
            }
            inlined.insert(vertex.clone(), ids);
        }
        Self {
            wiki,
            name: name.to_string(),
            order,
            inlined,
            owner,
        }
    }
    pub(super) fn name(&self) -> &str {
        &self.name
    }
    pub(super) fn node(&self, id: &str) -> Option<&Shown> {
        self.wiki.get(id)
    }
    pub(super) fn all_ids(&self) -> &[String] {
        &self.wiki.order
    }
    /// The edges of one node that carry a given name, both directions.
    pub(super) fn edges(&self, id: &str, name: &str) -> Vec<&EdgeLine> {
        self.wiki.get(id).map_or_else(Vec::new, |node| {
            node.edges.iter().filter(|edge| edge.name == name).collect()
        })
    }
    /// The page a node is read in: its own for a vertex, else the first vertex
    /// that expands it; `None` for another scope or a node no page shows.
    pub(super) fn page_of(&self, id: &str) -> Option<String> {
        let node = self.wiki.get(id)?;
        if node.scope.as_deref() != Some(self.name.as_str()) {
            return None;
        }
        if is_vertex(node.kind) {
            return Some(format!("{id}.md"));
        }
        self.owner.get(id).map(|vertex| format!("{vertex}.md"))
    }
    /// The other vertices that expand this node, in repository order.
    pub(super) fn shared(&self, id: &str, here: &str) -> Vec<String> {
        self.order
            .iter()
            .filter(|vertex| format!("{vertex}.md") != here)
            .filter(|vertex| {
                self.inlined
                    .get(*vertex)
                    .is_some_and(|ids| ids.iter().any(|shown| shown == id))
            })
            .cloned()
            .collect()
    }
    /// A reference to a node: a link to its page and anchor, or plain text.
    pub(super) fn link(&self, id: &str, here: &str) -> String {
        let title = self
            .wiki
            .get(id)
            .map_or_else(|| id.to_string(), |node| node.title.clone());
        match self.page_of(id) {
            None => {
                let outside = match self.wiki.get(id).and_then(|node| node.scope.clone()) {
                    Some(scope) if scope != self.name => format!(" *(in {scope})*"),
                    _ => String::new(),
                };
                format!("`{id}` {title}{outside}")
            }
            Some(page) if page == here => format!("`{id}` {title}"),
            Some(page) => format!("[`{id}` {title}]({page}{})", anchor(self.wiki.get(id))),
        }
    }
    /// A page's opening: the front matter, then the title.
    pub(super) fn page_head(&self, node: &Shown) -> String {
        format!("{}# {}\n\n", front_matter(node), node.title)
    }
    /// The text with its narrowed passages marked the Markdown way (d-bde9).
    pub(super) fn marked(&self, node: &Shown, text: &str) -> String {
        match &node.cancellation {
            Some(retraction) => markdown_retracted(text, retraction),
            None => text.to_string(),
        }
    }
}

fn in_scope(wiki: &Wiki, id: &str, name: &str) -> bool {
    wiki.get(id)
        .is_some_and(|node| node.scope.as_deref() == Some(name))
}

/// The nodes a vertex's page expands in full, in the page's section order.
fn inlined_by(wiki: &Wiki, vertex: &Shown, name: &str) -> Vec<String> {
    let mut ids = Vec::new();
    match vertex.kind {
        NodeKind::Need => {
            ids.extend(edge_ids(wiki, &vertex.id, "targets"));
            let filed = edge_ids(wiki, &vertex.id, "filed-as");
            for requirement in &filed {
                ids.extend(edge_ids(wiki, requirement, "raised"));
            }
            ids.extend(filed);
            ids.extend(edge_ids(wiki, &vertex.id, "waits-on"));
        }
        NodeKind::Decision => ids.extend(edge_ids(wiki, &vertex.id, "closed-by")),
        _ => {}
    }
    let mut seen = BTreeSet::new();
    ids.retain(|id| in_scope(wiki, id, name) && is_inline(wiki.get(id).unwrap().kind));
    ids.retain(|id| seen.insert(id.clone()));
    ids
}

fn edge_ids(wiki: &Wiki, id: &str, name: &str) -> Vec<String> {
    wiki.get(id).map_or_else(Vec::new, |node| {
        node.edges
            .iter()
            .filter(|edge| edge.name == name)
            .map(|edge| edge.to.clone())
            .collect()
    })
}

/// The anchor a link to an inline node uses; a vertex page needs none.
fn anchor(node: Option<&Shown>) -> String {
    match node {
        Some(node) if !is_vertex(node.kind) => format!("#{}", node.id.replace('-', "")),
        _ => String::new(),
    }
}

/// The node's own facts as front matter, which GitHub renders as a table.
fn front_matter(node: &Shown) -> String {
    let mut out = String::from("---\n");
    out += &format!("id: {}\n", node.id);
    out += &format!("kind: {}\n", node.kind.name());
    if let Some(state) = state(node) {
        out += &format!("state: {}\n", quote(&state));
    }
    if let Some(scope) = &node.scope {
        out += &format!("scope: {}\n", quote(scope));
    }
    if let Some(created) = &node.created {
        out += &format!("created: {}\n", created.get(..10).unwrap_or(created));
    }
    for (name, ids) in grouped_edges(node) {
        out += &format!("{}: [{}]\n", name.replace('-', "_"), ids.join(", "));
    }
    out += "---\n\n";
    out
}

fn state(node: &Shown) -> Option<String> {
    match &node.data {
        NodeData::Need(_) => Some(node.need.unwrap_or("open").to_string()),
        NodeData::Requirement(data) => Some(data.state.name().to_string()),
        _ => None,
    }
}

/// The node's edges grouped by name, in the order they first appear.
fn grouped_edges(node: &Shown) -> Vec<(String, Vec<String>)> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for edge in &node.edges {
        match groups.iter_mut().find(|(name, _)| *name == edge.name) {
            Some((_, ids)) => ids.push(edge.to.clone()),
            None => groups.push((edge.name.clone(), vec![edge.to.clone()])),
        }
    }
    groups
}

/// A YAML scalar: bare when it is a plain word, else double-quoted.
fn quote(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "_.-".contains(ch));
    if plain {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}

/// The view's `[[retracted by <id>: <mark>]]` written the way GitHub renders
/// it: `~~<mark>~~ *(retracted by <id>)*`, longest passage first (d-bde9).
fn markdown_retracted(text: &str, retraction: &Retraction) -> String {
    let mut marks: Vec<&Narrowed> = retraction
        .narrowed
        .iter()
        .filter(|narrow| !narrow.mark.is_empty())
        .collect();
    if marks.is_empty() {
        return text.to_string();
    }
    marks.sort_by_key(|narrow| std::cmp::Reverse(narrow.mark.len()));
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    'scan: while !rest.is_empty() {
        for narrow in &marks {
            if rest.starts_with(&narrow.mark) {
                out.push_str(&format!(
                    "~~{}~~ *(retracted by {})*",
                    narrow.mark, narrow.by
                ));
                rest = &rest[narrow.mark.len()..];
                continue 'scan;
            }
        }
        let ch = rest.chars().next().expect("rest is not empty");
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}
