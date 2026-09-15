//! Reading a 0.4 ledger into an intermediate form. This is the only file of
//! the crate that touches gy-core's string keys (N-65).
use gy_ledger::{Error, Result};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

/// The forward relationship keys a 0.4 node stores.
const RELATIONS: &[&str] = &[
    "targets",
    "spawned-by",
    "filed-as",
    "depends-on",
    "relies-on",
    "raised",
    "closes",
    "narrows",
    "widens",
    "supersedes",
    "completes",
    "measured-by",
];

/// Attributes the migration drops (a publication keeps them in a later step).
const DROPPED: &[&str] = &[
    "imported",
    "legacy_status",
    "source_file",
    "source_line",
    "source_section",
    "bearer_count",
    "acceptance",
    "parent_issue",
];

/// The 0.4 ledger, reduced to plain data.
#[derive(Debug, Clone, Default)]
pub struct Legacy {
    pub nodes: Vec<LegacyNode>,
}

/// One 0.4 node: identity and body, plus its attributes and edges as data.
/// The identity fields are read by the node transfer in the next step (N-67).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LegacyNode {
    pub kind: String,
    pub id: String,
    pub scope: String,
    pub created: String,
    pub title: String,
    pub body: String,
    /// Scalar attributes as text.
    pub attrs: BTreeMap<String, String>,
    /// List attributes (ids or words).
    pub lists: BTreeMap<String, Vec<String>>,
    /// Relationship name to target ids.
    pub links: BTreeMap<String, Vec<String>>,
}

pub fn read(dir: &Path) -> Result<Legacy> {
    let store = gy_core::Store::open(dir).map_err(|error| Error::invalid(error.message))?;
    let nodes = store.nodes.values().map(LegacyNode::of).collect();
    Ok(Legacy { nodes })
}

impl LegacyNode {
    fn of(node: &gy_core::Node) -> Self {
        let mut attrs = BTreeMap::new();
        let mut lists = BTreeMap::new();
        for (key, value) in &node.attrs {
            if value.is_array() {
                lists.insert(key.clone(), gy_core::ids(Some(value)));
            } else if !value.is_object() {
                attrs.insert(key.clone(), gy_core::text(value));
            }
        }
        let mut links = BTreeMap::new();
        for relation in RELATIONS {
            let targets = node.refs(relation);
            if !targets.is_empty() {
                links.insert((*relation).to_string(), targets);
            }
        }
        Self {
            kind: node.kind().to_string(),
            id: node.id().to_string(),
            scope: node.scope().to_string(),
            created: node.get("created").to_string(),
            title: node.get("title").to_string(),
            body: node.body.clone(),
            attrs,
            lists,
            links,
        }
    }

    /// The kind this node becomes: a gate becomes a need (D-68).
    pub fn new_kind(&self) -> &str {
        if self.kind == "gate" {
            "need"
        } else {
            &self.kind
        }
    }

    /// A decision whose `decision_scope` was never recorded.
    pub fn has_unrecorded_scope(&self) -> bool {
        self.kind == "decision"
            && self
                .attrs
                .get("decision_scope")
                .is_none_or(|text| text.trim().is_empty())
    }

    fn dropped(&self) -> Vec<String> {
        DROPPED
            .iter()
            .filter(|name| self.attrs.contains_key(**name))
            .map(|name| (*name).to_string())
            .collect()
    }
}

/// What the read found, before anything is written.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub before: BTreeMap<String, usize>,
    pub after: BTreeMap<String, usize>,
    pub aliases: usize,
    pub unrecorded: usize,
    pub waiting_on: usize,
    pub links: usize,
    pub dropped: Vec<String>,
}

impl Legacy {
    pub fn report(&self) -> Report {
        let mut report = Report::default();
        for node in &self.nodes {
            *report.before.entry(node.kind.clone()).or_default() += 1;
            *report.after.entry(node.new_kind().to_string()).or_default() += 1;
            report.aliases += 1;
            if node.has_unrecorded_scope() {
                report.unrecorded += 1;
            }
            report.waiting_on += node.lists.get("waiting-on").map_or(0, Vec::len);
            report.links += node.links.values().map(Vec::len).sum::<usize>();
            for name in node.dropped() {
                if !report.dropped.contains(&name) {
                    report.dropped.push(name);
                }
            }
        }
        report.dropped.sort();
        report
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "before: {}", counts(&self.before))?;
        writeln!(f, "after: {}", counts(&self.after))?;
        writeln!(f, "aliases: {}", self.aliases)?;
        writeln!(f, "unrecorded decision scope: {}", self.unrecorded)?;
        writeln!(f, "waiting-on ids: {}", self.waiting_on)?;
        writeln!(f, "edges: {}", self.links)?;
        write!(f, "dropped attributes: {}", self.dropped.join(", "))
    }
}

fn counts(map: &BTreeMap<String, usize>) -> String {
    map.iter()
        .map(|(kind, count)| format!("{kind} {count}"))
        .collect::<Vec<_>>()
        .join(", ")
}
