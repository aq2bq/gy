//! req cancel: cancel a requirement that was not done, recording the reason
//! and where it came from (D-70).
use super::{Operation, Outcome, Repository, advice_for, node_of, now};
use crate::model::{Cancellation, NodeData, NodeId, NodeKind, RequirementState};
use crate::store::{Error, Result, Store};

#[derive(Clone)]
pub struct ReqCancel {
    pub id: NodeId,
    pub reason: String,
    pub source: String,
}
impl<S: Store> Operation<S> for ReqCancel {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.reason.trim().is_empty() || self.source.trim().is_empty() {
            return Err(Error::invalid("cancelling needs a reason and a source"));
        }
        let mut node = node_of(repo, &self.id, NodeKind::Requirement)?;
        node.advance(RequirementState::Cancelled)?;
        if let NodeData::Requirement(data) = node.data_mut() {
            data.cancellation = Some(Cancellation::new(
                self.reason.clone(),
                self.source.clone(),
                now(),
            )?);
        }
        let why = format!("req cancel {}", self.id);
        repo.transaction(&why, &self.source, |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed: vec!["cancelled".into()],
            missing,
            next,
            unresolved: Vec::new(),
            value: self.id,
        })
    }
}
