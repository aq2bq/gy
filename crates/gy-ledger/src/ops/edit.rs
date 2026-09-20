//! edit: change a node's title, body, or free attributes, and never its state,
//! edges, or creation (proposal-v3 3). A reason is required, and the reserved
//! names of protected concepts are refused. `--set scope=<name>` moves the node
//! to a scope gy.toml declares (n-1aad); `--set decision_scope=<text>` fills a
//! decision's unrecorded scope once, and `--set key=` drops a free attribute
//! (n-79fb).
use super::{Operation, Outcome, Repository, marks};
use crate::model::{DecisionScope, Edge, Node, NodeData, NodeId};
use crate::store::{Error, Result, Store};

#[derive(Clone)]
pub struct Edit {
    pub id: NodeId,
    pub reason: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub set: Vec<(String, String)>,
    pub append: Vec<(String, String)>,
}
impl<S: Store> Operation<S> for Edit {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.reason.trim().is_empty() {
            return Err(Error::invalid("editing needs a reason"));
        }
        if self.title.is_none()
            && self.body.is_none()
            && self.set.is_empty()
            && self.append.is_empty()
        {
            return Err(Error::invalid(
                "editing changes at least one of title, body, set, append",
            ));
        }
        for (key, _) in self.set.iter().chain(&self.append) {
            if reserved(key) {
                return Err(Error::invalid(format!(
                    "{key} is not editable; change it with its own operation"
                )));
            }
        }
        let mut node = repo
            .get(&self.id)?
            .ok_or_else(|| Error::invalid(format!("{} does not exist", self.id)))?;
        let watched = watched(&node, repo.incoming(&self.id)?);
        let changed = apply(&mut node, &self, repo.scopes())?;
        let unresolved = unresolved_names(watched, &node);
        let why = format!("edit {}", self.id);
        repo.transaction(&why, &self.reason, |repo| repo.put(&node))?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed,
            missing: Vec::new(),
            next: Vec::new(),
            unresolved,
            value: self.id,
        })
    }
}

/// The incoming `narrows` / `supersedes` marks that resolve in `older` as it
/// stands before the edit: the passages an edit could leave unresolved. One
/// pass over the graph gives them all (n-847d).
fn watched(older: &Node, incoming: Vec<Edge>) -> Vec<(String, &'static str, String)> {
    incoming
        .into_iter()
        .filter(|edge| marks::required(edge.label))
        .filter_map(|edge| {
            let mark = edge.mark?;
            marks::resolves(older, &mark).then_some((edge.to.to_string(), edge.label.name(), mark))
        })
        .collect()
}

/// The watched marks that no longer resolve in the edited node, named for the
/// writer: `<decision> <relation>: "<mark>"`.
fn unresolved_names(watched: Vec<(String, &'static str, String)>, node: &Node) -> Vec<String> {
    watched
        .into_iter()
        .filter(|(_, _, mark)| !marks::resolves(node, mark))
        .map(|(by, relation, mark)| format!("{by} {relation}: {mark:?}"))
        .collect()
}

fn apply(node: &mut Node, edit: &Edit, scopes: &[String]) -> Result<Vec<String>> {
    let mut changed = Vec::new();
    if let Some(title) = &edit.title {
        node.set_title(title.clone())?;
        changed.push("title".into());
    }
    if let Some(body) = &edit.body {
        node.set_body(body.clone());
        changed.push("body".into());
    }
    for (key, value) in &edit.set {
        set_one(node, key, value, scopes, &mut changed)?;
    }
    for (key, value) in &edit.append {
        append_one(node, key, value, &mut changed)?;
    }
    Ok(changed)
}

/// One `--set`: the typed keys `scope` and `decision_scope`, or a free
/// attribute. An empty value drops a free attribute.
fn set_one(
    node: &mut Node,
    key: &str,
    value: &str,
    scopes: &[String],
    changed: &mut Vec<String>,
) -> Result<()> {
    match key {
        "scope" => {
            if !scopes.iter().any(|scope| scope == value) {
                return Err(Error::invalid(format!(
                    "unknown scope {value}; expected one of {}",
                    scopes.join(", ")
                )));
            }
            node.set_scope(value)?;
            changed.push("scope".into());
        }
        "decision_scope" => {
            record_scope(node, value)?;
            changed.push("decision_scope".into());
        }
        _ if value.is_empty() => {
            if node.remove_free(key).is_some() {
                changed.push(format!("free:{key}"));
            }
        }
        _ => {
            node.set_free(key, value);
            changed.push(format!("free:{key}"));
        }
    }
    Ok(())
}

/// Fill a decision's unrecorded applicability scope, once (n-79fb).
fn record_scope(node: &mut Node, text: &str) -> Result<()> {
    let scope = DecisionScope::recorded(text)?;
    match node.data_mut() {
        NodeData::Decision(data) => {
            if !data.scope.is_unrecorded() {
                return Err(Error::invalid(
                    "the decision already has a recorded scope; use narrows or supersedes",
                ));
            }
            data.scope = scope;
            Ok(())
        }
        _ => Err(Error::invalid("only a decision has an applicability scope")),
    }
}

fn append_one(node: &mut Node, key: &str, value: &str, changed: &mut Vec<String>) -> Result<()> {
    if key == "scope" || key == "decision_scope" {
        return Err(Error::invalid(format!("{key} is set, not appended")));
    }
    if value.is_empty() {
        return Err(Error::invalid(format!("appending to {key} needs a value")));
    }
    node.append_free(key, value);
    changed.push(format!("free:{key}"));
    Ok(())
}

/// Names owned by typed fields or by another operation. `edit` writes only free
/// attributes and the two typed keys above, so these would write a parallel,
/// unread value.
const RESERVED: &[&str] = &[
    "id",
    "type",
    "kind",
    "created",
    "title",
    "body",
    "links",
    "state",
    "status",
    "reference",
    "ref",
    "approval",
    "revisions",
    "completion",
    "cancellation",
    "closed",
    "closure",
    "decider",
    "options",
    "satisfied",
    "satisfied_at",
];

fn reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}
