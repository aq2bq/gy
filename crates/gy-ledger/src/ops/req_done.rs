//! req done: record the evidence that an approved requirement shipped (D-70).
use super::{Operation, Outcome, Repository, advice_for, node_of, now};
use crate::model::{Completion, NodeData, NodeId, NodeKind, RequirementState};
use crate::store::{Error, Result, Store};

pub struct ReqDone {
    pub id: NodeId,
    pub evidence: String,
}
impl<S: Store> Operation<S> for ReqDone {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.evidence.trim().is_empty() {
            return Err(Error::invalid("completing needs evidence"));
        }
        let mut node = node_of(repo, &self.id, NodeKind::Requirement)?;
        node.advance(RequirementState::Done)?;
        if let NodeData::Requirement(data) = node.data_mut() {
            data.completion = Some(Completion::new(self.evidence.clone(), now())?);
        }
        let why = format!("req done {}", self.id);
        repo.transaction(&why, &self.evidence, |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed: vec!["done".into()],
            missing,
            next,
            value: self.id,
        })
    }
}
