//! criterion add: an acceptance criterion. A blank title is rejected by the
//! model.
use super::{Operation, Outcome, Repository, advice_for, today};
use crate::model::{Node, NodeId, NodeKind};
use crate::store::{Result, Store};

pub struct CriterionAdd {
    pub scope: String,
    pub title: String,
}
impl<S: Store> Operation<S> for CriterionAdd {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        let id = repo.next_id(NodeKind::Criterion)?;
        let node = Node::criterion(id.clone(), &self.scope, &today(), &self.title)?;
        let why = format!("criterion add {id}");
        repo.transaction(&why, "criterion add", |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(id.clone()),
            changed: vec!["created".into()],
            missing,
            next,
            value: id,
        })
    }
}
