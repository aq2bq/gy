//! The gate the operations install (n-557f): the model's rule wrapped in the
//! store's terms, and the body `gy_ledger::sync` now has.
use crate::model::{self, Node, NodeId, NodeKind};
use crate::store::{Error, Gate, Result, log};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

/// The gate a repository installs: it shows the previous state to the model as
/// nodes on demand, types the changes, and asks the model's rule.
#[derive(Debug, Default)]
pub struct Rules;

impl Gate for Rules {
    fn admit(&self, before: &BTreeMap<String, Value>, changes: &[log::Change]) -> Result<()> {
        let view = Nodes { nodes: before };
        let changes = typed(changes)?;
        model::admit(&view, &changes)
    }
}

/// `gy sync`'s body after n-557f: the store's sync with this build's rule.
/// `gy_ledger::sync` keeps its name and shape through the re-export in lib.rs.
pub fn sync(ledger: &Path, remote: &str) -> Result<crate::store::remote::Sync> {
    crate::store::remote::sync_with(ledger, remote, Arc::new(Rules))
}

/// The previous state as the model reads it. Only what the rule asks for is
/// turned into a node, so a commit decodes nothing it does not judge (n-557f).
struct Nodes<'a> {
    nodes: &'a BTreeMap<String, Value>,
}

impl model::Before for Nodes<'_> {
    fn get(&self, id: &NodeId) -> Option<Node> {
        node(self.nodes.get(&id.to_string())?)
    }
    /// The kind is read from the id prefix, so no value is parsed to filter.
    fn of_kind(&self, kind: NodeKind) -> Vec<Node> {
        let prefix = format!("{}-", kind.prefix());
        self.nodes
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .filter_map(|(_, value)| node(value))
            .collect()
    }
}

fn node(value: &Value) -> Option<Node> {
    serde_json::from_value(value.clone()).ok()
}

fn typed(changes: &[log::Change]) -> Result<Vec<model::Change>> {
    changes.iter().map(typed_change).collect()
}

fn typed_change(change: &log::Change) -> Result<model::Change> {
    let bad = || Error::invalid("a changed node is not a node");
    Ok(match change {
        log::Change::Created { value, .. } => model::Change::Created(node(value).ok_or_else(bad)?),
        log::Change::Updated { value, .. } => model::Change::Updated(node(value).ok_or_else(bad)?),
        log::Change::Deleted { value, .. } => {
            model::Change::Deleted(node(value).ok_or_else(bad)?.id().clone())
        }
        log::Change::ScopeRenamed { from, to, .. } => model::Change::ScopeRenamed {
            from: from.clone(),
            to: to.clone(),
        },
    })
}
