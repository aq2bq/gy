//! question add: a question with a decider and at least two options.
use super::{Operation, Outcome, Repository, advice_for, today};
use crate::model::{Node, NodeData, NodeId, NodeKind};
use crate::store::{Error, Result, Store};

pub struct QuestionAdd {
    pub scope: String,
    pub title: String,
    pub decider: String,
    pub options: Vec<String>,
    pub body: Option<String>,
}
impl<S: Store> Operation<S> for QuestionAdd {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.decider.trim().is_empty() {
            return Err(Error::invalid("a question needs a decider"));
        }
        if self.options.len() < 2 {
            return Err(Error::invalid("a question needs at least two options"));
        }
        let id = repo.next_id(NodeKind::Question)?;
        let mut node = Node::question(id.clone(), &self.scope, &today(), &self.title)?;
        if let NodeData::Question(data) = node.data_mut() {
            data.decider = Some(self.decider);
            data.options = self.options;
        }
        if let Some(body) = &self.body {
            node.set_body(body.clone());
        }
        let why = format!("question add {id}");
        repo.transaction(&why, "question add", |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(id.clone()),
            changed: vec!["created".into(), "decider".into(), "options".into()],
            missing,
            next,
            value: id,
        })
    }
}
