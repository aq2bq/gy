//! criterion add: an acceptance criterion. A blank title is rejected by the
//! model.
use super::{Operation, Outcome, Repository, advice_for, now};
use crate::model::{Node, NodeId, NodeKind};
use crate::store::{Result, Store};

#[derive(Clone)]
pub struct CriterionAdd {
    pub scope: String,
    pub title: String,
    pub body: Option<String>,
}
impl<S: Store> Operation<S> for CriterionAdd {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        let id = repo.next_id(NodeKind::Criterion)?;
        let mut node = Node::criterion(id.clone(), &self.scope, &now(), &self.title)?;
        if let Some(body) = &self.body {
            node.set_body(body.clone());
        }
        let why = format!("criterion add {id}");
        repo.transaction(&why, "criterion add", |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(id.clone()),
            changed: vec!["created".into()],
            missing,
            next,
            unresolved: Vec::new(),
            value: id,
        })
    }
}
