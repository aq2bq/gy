//! One 0.4 node as plain data, and the typed accessors the transfer reads.
use super::links::{LegacyLink, RELATIONS, edges_of};
use std::collections::BTreeMap;

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

/// The 0.4 requirement attributes that are frozen as a publication (N-69).
const RECORDS: &[&str] = &[
    "transitions",
    "record_history",
    "deviations",
    "artifacts",
    "cleanup_done",
    "data_migration",
    "quality_gates",
    "implementation_report",
    "dispatch",
];

/// One recorded state change of a 0.4 requirement.
#[derive(Debug, Clone, Default)]
pub struct LegacyTransition {
    pub to: String,
    pub evidence: Option<String>,
    pub at: Option<String>,
}

/// One 0.4 node: identity and body, plus its attributes, edges, and transitions
/// as data.
#[derive(Debug, Clone)]
pub struct LegacyNode {
    pub kind: String,
    pub id: String,
    pub scope: String,
    pub created: String,
    pub title: String,
    pub body: String,
    /// The original file text, for the frozen publication.
    pub raw: String,
    /// Scalar attributes as text.
    pub attrs: BTreeMap<String, String>,
    /// List attributes (ids or words).
    pub lists: BTreeMap<String, Vec<String>>,
    /// Relationship name to its edges.
    pub links: BTreeMap<String, Vec<LegacyLink>>,
    pub transitions: Vec<LegacyTransition>,
}

impl LegacyNode {
    pub(super) fn of(node: &gy_core::Node) -> Self {
        let (attrs, lists) = attribute_bags(node);
        let kind = node.kind().to_string();
        let links = links_of(node, &kind);
        let transitions = node
            .attrs
            .get("transitions")
            .map(transitions_of)
            .unwrap_or_default();
        Self {
            kind,
            id: node.id().to_string(),
            scope: node.scope().to_string(),
            created: node.get("created").to_string(),
            title: node.get("title").to_string(),
            body: node.body.clone(),
            raw: std::fs::read_to_string(&node.path).unwrap_or_default(),
            attrs,
            lists,
            links,
            transitions,
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

    /// The forward edges except `measured-by` (a gate's, written as waits-on).
    pub fn out_edges(&self) -> impl Iterator<Item = (&str, &[LegacyLink])> {
        self.links
            .iter()
            .filter(|(name, _)| name.as_str() != "measured-by")
            .map(|(name, links)| (name.as_str(), links.as_slice()))
    }

    pub fn measured_by(&self) -> &[LegacyLink] {
        self.link("measured-by")
    }

    pub fn decision_scope(&self) -> Option<&str> {
        self.text("decision_scope")
    }

    /// The status without a trailing variant suffix, as gy-core reads it.
    pub fn status(&self) -> &str {
        let status = self.attrs.get("status").map(String::as_str).unwrap_or("");
        status
            .split_once(" (")
            .filter(|(_, tail)| tail.ends_with(')'))
            .map_or(status, |(base, _)| base)
    }

    pub fn decider(&self) -> Option<&str> {
        self.text("decider")
    }

    pub fn options(&self) -> &[String] {
        self.list("options")
    }

    pub fn closed_by(&self) -> Option<&str> {
        self.text("closed_by")
    }

    pub fn closure_note(&self) -> Option<&str> {
        self.text("closure_note")
    }

    pub fn closed_at(&self) -> Option<&str> {
        self.text("closed_at")
    }

    pub fn satisfied(&self) -> bool {
        self.text("satisfied") == Some("true")
    }

    pub fn satisfied_at(&self) -> Option<&str> {
        self.text("satisfied_at")
    }

    pub fn evidence(&self) -> Option<&str> {
        self.text("evidence")
    }

    pub fn dropped_by(&self) -> Option<&str> {
        self.text("dropped_by")
    }

    pub fn dropped_reason(&self) -> Option<&str> {
        self.text("dropped_reason")
    }

    pub fn next_evidence(&self) -> Option<&str> {
        self.text("next_evidence")
    }

    pub fn responsible(&self) -> Option<&str> {
        self.text("responsible")
    }

    pub fn remaining_work(&self) -> Option<&str> {
        self.text("remaining_work")
    }

    pub fn residual(&self) -> Option<&str> {
        self.text("residual")
    }

    pub fn unresolved(&self) -> Option<&str> {
        self.text("unresolved")
    }

    /// The owning nodes of a 0.4 question: a need means it waits on the
    /// question (d-63f8).
    pub fn belongs_to(&self) -> &[String] {
        self.list("belongs-to")
    }

    pub fn pr_url(&self) -> Option<&str> {
        self.text("pr_url")
    }

    pub fn waiting_on(&self) -> &[String] {
        self.list("waiting-on")
    }

    /// The evidence or time of the transition whose `to` is `to`.
    pub fn transition_evidence(&self, to: &str) -> Option<&str> {
        self.transitions
            .iter()
            .find(|step| step.to == to)
            .and_then(|step| step.evidence.as_deref())
    }

    pub fn transition_at(&self, to: &str) -> Option<&str> {
        self.transitions
            .iter()
            .find(|step| step.to == to)
            .and_then(|step| step.at.as_deref())
    }

    /// A decision whose `decision_scope` was never recorded.
    pub fn has_unrecorded_scope(&self) -> bool {
        self.kind == "decision" && self.decision_scope().is_none()
    }

    /// A requirement that carries records to freeze (N-69).
    pub fn has_records(&self) -> bool {
        self.kind == "requirement"
            && self.attrs.keys().chain(self.lists.keys()).any(|name| {
                RECORDS.contains(&name.as_str())
                    || name.starts_with("production_")
                    || name.starts_with("pr_")
            })
    }

    pub(super) fn dropped(&self) -> Vec<String> {
        DROPPED
            .iter()
            .filter(|name| self.attrs.contains_key(**name))
            .map(|name| (*name).to_string())
            .collect()
    }

    pub(super) fn link(&self, name: &str) -> &[LegacyLink] {
        self.links.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    fn text(&self, name: &str) -> Option<&str> {
        self.attrs
            .get(name)
            .map(String::as_str)
            .filter(|text| !text.is_empty())
    }

    fn list(&self, name: &str) -> &[String] {
        self.lists.get(name).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Split the attribute map into scalar text and list values.
fn attribute_bags(
    node: &gy_core::Node,
) -> (BTreeMap<String, String>, BTreeMap<String, Vec<String>>) {
    let mut attrs = BTreeMap::new();
    let mut lists = BTreeMap::new();
    for (key, value) in &node.attrs {
        if value.is_array() {
            lists.insert(key.clone(), gy_core::ids(Some(value)));
        } else if !value.is_object() {
            attrs.insert(key.clone(), gy_core::text(value));
        }
    }
    (attrs, lists)
}

/// The forward edges, leaving out the decision side's copy of `closes` (0.4
/// stores it on both sides).
fn links_of(node: &gy_core::Node, kind: &str) -> BTreeMap<String, Vec<LegacyLink>> {
    let mut links = BTreeMap::new();
    for relation in RELATIONS {
        if *relation == "closes" && kind == "decision" {
            continue;
        }
        let edges = node.attrs.get(*relation).map(edges_of).unwrap_or_default();
        if !edges.is_empty() {
            links.insert((*relation).to_string(), edges);
        }
    }
    links
}

fn transitions_of(value: &serde_json::Value) -> Vec<LegacyTransition> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let map = item.as_object()?;
            Some(LegacyTransition {
                to: map.get("to").map(gy_core::text).unwrap_or_default(),
                evidence: map
                    .get("evidence")
                    .map(gy_core::text)
                    .filter(|text| !text.is_empty()),
                at: map
                    .get("at")
                    .map(gy_core::text)
                    .filter(|text| !text.is_empty()),
            })
        })
        .collect()
}
