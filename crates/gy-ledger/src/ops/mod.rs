//! Operations: one intent = one command = one transaction (D-69, D-75). Each
//! operation is its own type implementing `Operation`.
pub mod config;
pub mod repository;

pub use repository::Repository;

use crate::model::{Link, Node, NodeData, NodeId, NodeKind, Relation};
use crate::store::{Error, Result, Store};

/// What an operation produced: the node it touched, what it changed, what is
/// still missing, and the operations that could follow (N-39).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome<T> {
    pub id: Option<NodeId>,
    pub changed: Vec<String>,
    pub missing: Vec<String>,
    pub next: Vec<String>,
    pub value: T,
}

/// One intent, run against a repository (D-69).
pub trait Operation<S: Store> {
    type Output;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>>;
}

/// A sample operation that creates a need targeting existing criteria. N-37
/// replaces it with the full set; this exists to prove the frame.
pub struct NeedAdd {
    pub scope: String,
    pub title: String,
    pub targets: Vec<NodeId>,
}
impl<S: Store> Operation<S> for NeedAdd {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.targets.is_empty() {
            return Err(Error::invalid("a need needs at least one target"));
        }
        for target in &self.targets {
            match repo.get(target)? {
                Some(node) if node.kind() == NodeKind::Criterion => {}
                Some(_) => return Err(Error::invalid(format!("{target} is not a criterion"))),
                None => return Err(Error::invalid(format!("{target} does not exist"))),
            }
        }
        let id = repo.next_id(NodeKind::Need)?;
        let mut node = Node::need(
            id.clone(),
            self.scope.as_str(),
            &today(),
            self.title.as_str(),
        )?;
        for target in &self.targets {
            node.link(Link::new(id.clone(), Relation::Targets, target.clone())?);
        }
        if let NodeData::Need(data) = node.data_mut() {
            data.targets = self.targets.clone();
        }
        repo.transaction("need add", "operation", |repo| repo.put(&node))?;
        Ok(Outcome {
            id: Some(id.clone()),
            changed: vec![
                "title".into(),
                "scope".into(),
                "created".into(),
                "targets".into(),
            ],
            missing: Vec::new(),
            next: Vec::new(),
            value: id,
        })
    }
}

fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}
