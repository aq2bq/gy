//! The one rebase exception: concurrent, single-edge narrows additions to the
//! same pair. Track each original history, never infer intent from its tip.
use super::rules::{prefix, touched};
use crate::store::{log, replay};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

type Nodes = BTreeMap<String, Value>;
type Pairs = BTreeMap<String, Option<String>>;

pub(super) struct Narrows {
    local: Nodes,
    remote_pairs: Pairs,
    local_pairs: Pairs,
}

impl Narrows {
    pub(super) fn new(base: Nodes) -> Self {
        Self {
            local: base,
            remote_pairs: Pairs::new(),
            local_pairs: Pairs::new(),
        }
    }

    pub(super) fn remote(&mut self, nodes: &Nodes, event: &log::Event) {
        remember(
            &mut self.remote_pairs,
            nodes,
            event,
            addition(nodes, event).as_ref(),
        );
    }

    /// Always advance the original local history, even for a rejected or
    /// already-landed row. Only divergent rows constrain the exception.
    pub(super) fn place(
        &mut self,
        nodes: &Nodes,
        event: &log::Event,
        landed: bool,
    ) -> Option<(log::Event, String)> {
        let addition = addition(&self.local, event);
        if !landed {
            remember(&mut self.local_pairs, &self.local, event, addition.as_ref());
        }
        replay::apply(&mut self.local, event);
        let added = addition.filter(|added| {
            !landed
                && self.remote_pairs.get(&added.from) == Some(&Some(added.to.clone()))
                && self.local_pairs.get(&added.from) == Some(&Some(added.to.clone()))
        })?;
        let mut value: Links = serde_json::from_value(nodes.get(&added.from)?.clone()).ok()?;
        value.links.push(added.edge);
        let mut placed = event.clone();
        placed.changes = vec![log::Change::Updated {
            node: added.from.clone(),
            value: serde_json::to_value(value).ok()?,
        }];
        Some((placed, added.from))
    }
}

/// None means this source had a touch outside the same-pair additions. It
/// stays blocked even if a later edit restores the earlier value.
fn remember(pairs: &mut Pairs, nodes: &Nodes, event: &log::Event, added: Option<&Addition>) {
    for id in touched(nodes, event) {
        let target = added
            .filter(|added| added.from == id)
            .map(|added| added.to.clone());
        pairs
            .entry(id)
            .and_modify(|held| {
                if *held != target {
                    *held = None;
                }
            })
            .or_insert(target);
    }
}

#[derive(Serialize, Deserialize)]
struct Links {
    links: Vec<Value>,
    #[serde(flatten)]
    rest: Map<String, Value>,
}

#[derive(Deserialize)]
struct Id {
    kind: String,
    hash: String,
}

impl Id {
    fn text(&self) -> Option<String> {
        Some(format!("{}-{}", prefix(&self.kind)?, self.hash))
    }
}

#[derive(Deserialize)]
struct Edge {
    from: Id,
    to: Id,
    label: String,
    reversed: bool,
    mark: Option<String>,
}

struct Addition {
    from: String,
    to: String,
    edge: Value,
}

fn addition(nodes: &Nodes, event: &log::Event) -> Option<Addition> {
    let [log::Change::Updated { node, value }] = event.changes.as_slice() else {
        return None;
    };
    if event.source != "link" {
        return None;
    }
    let mut after: Links = serde_json::from_value(value.clone()).ok()?;
    let edge = after.links.pop()?;
    let shape: Edge = serde_json::from_value(edge.clone()).ok()?;
    let target = shape.to.text()?;
    if shape.label != "Narrows"
        || shape.reversed
        || shape.mark.as_deref().is_none_or(str::is_empty)
        || shape.from.kind != "Decision"
        || shape.to.kind != "Decision"
        || shape.from.text().as_ref() != Some(node)
        || event.why != format!("link {node} narrows {target}")
        || serde_json::to_value(after).ok()?.ne(nodes.get(node)?)
    {
        return None;
    }
    Some(Addition {
        from: node.clone(),
        to: target,
        edge,
    })
}
