//! req done: record the evidence that an approved requirement shipped (D-70).
use super::{Operation, Outcome, Repository, node_of, outcome, today};
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
            data.completion = Some(Completion {
                evidence: self.evidence.clone(),
                at: today(),
            });
        }
        let why = format!("req done {}", self.id);
        repo.transaction(&why, &self.evidence, |repo| repo.put(&node))?;
        Ok(outcome(&self.id, "done"))
    }
}
