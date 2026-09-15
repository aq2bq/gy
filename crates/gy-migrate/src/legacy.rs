//! Reading a 0.4 ledger into an intermediate form. This module (with its
//! `node` and `links` children) is the only place that touches gy-core's
//! string keys: every key becomes a typed accessor.
pub(crate) mod free;
mod links;
mod node;

use crate::report::Report;
use gy_ledger::{Error, Result};
pub use links::LegacyLink;
pub use node::LegacyNode;
use std::collections::BTreeSet;
use std::path::Path;

/// The 0.4 requirement states, for the standard-outside report.
const REQUIREMENT_STATES: &[&str] = &[
    "unfiled",
    "defining",
    "awaiting-design",
    "awaiting-approval",
    "awaiting-implementation",
    "awaiting-audit",
    "awaiting-pr",
    "awaiting-merge",
    "awaiting-production",
    "awaiting-cleanup",
    "complete",
];

/// The 0.4 ledger, reduced to plain data.
#[derive(Debug, Clone, Default)]
pub struct Legacy {
    pub nodes: Vec<LegacyNode>,
}

pub fn read(dir: &Path) -> Result<Legacy> {
    let store = gy_core::Store::open(dir).map_err(|error| Error::invalid(error.message))?;
    let nodes = store.nodes.values().map(LegacyNode::of).collect();
    Ok(Legacy { nodes })
}

impl Legacy {
    pub fn by_id(&self, id: &str) -> Option<&LegacyNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Needs whose filed requirements are all complete: their state is derived
    /// from those requirements, so the transfer must not freeze them closed
    /// (D-28).
    pub fn derived_ids(&self) -> BTreeSet<String> {
        self.nodes
            .iter()
            .filter(|node| {
                let filed = node.link("filed-as");
                !filed.is_empty()
                    && filed.iter().all(|link| {
                        self.by_id(&link.id)
                            .is_some_and(|node| node.status() == "complete")
                    })
            })
            .map(|node| node.id.clone())
            .collect()
    }

    pub fn report(&self) -> Report {
        let mut report = Report::default();
        for node in &self.nodes {
            *report.before.entry(node.kind.clone()).or_default() += 1;
            *report.after.entry(node.new_kind().to_string()).or_default() += 1;
            report.aliases += 1;
            if node.has_unrecorded_scope() {
                report.unrecorded += 1;
            }
            if node.kind == "requirement" && !REQUIREMENT_STATES.contains(&node.status()) {
                report
                    .unmapped
                    .push(format!("{} {}", node.id, node.status()));
            }
            report.waiting_on += node.waiting_on().len();
            report.links += node.links.values().map(Vec::len).sum::<usize>();
            for (name, _) in free::attributes(node) {
                *report.free.entry(name).or_default() += 1;
            }
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
