//! edit: change a node's title, body, or free attributes, and never its state,
//! edges, scope, or creation (proposal-v3 3). A reason is required, and the
//! reserved names of protected concepts are refused.
use super::{Operation, Outcome, Repository};
use crate::model::{Node, NodeId};
use crate::store::{Error, Result, Store};

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
        let changed = apply(&mut node, &self)?;
        let why = format!("edit {}", self.id);
        repo.transaction(&why, &self.reason, |repo| repo.put(&node))?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed,
            missing: Vec::new(),
            next: Vec::new(),
            value: self.id,
        })
    }
}

fn apply(node: &mut Node, edit: &Edit) -> Result<Vec<String>> {
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
        node.set_free(key.clone(), value.clone());
        changed.push(format!("free:{key}"));
    }
    for (key, value) in &edit.append {
        node.append_free(key, value);
        changed.push(format!("free:{key}"));
    }
    Ok(changed)
}

/// Names owned by typed fields or by another operation. `edit` only writes
/// free attributes, so these keys would write a parallel, unread value.
const RESERVED: &[&str] = &[
    "id",
    "type",
    "kind",
    "scope",
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
    "decision_scope",
    "satisfied",
    "satisfied_at",
];

fn reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}
