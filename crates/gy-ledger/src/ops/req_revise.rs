//! req revise: send an approved requirement back to filed, keeping the reason
//! and its source as a revision record (D-70).
use super::{Operation, Outcome, Repository, advice_for, node_of, now};
use crate::model::{NodeData, NodeId, NodeKind, RequirementState, Revision};
use crate::store::{Error, Result, Store};

#[derive(Clone)]
pub struct ReqRevise {
    pub id: NodeId,
    pub reason: String,
    pub source: String,
}
impl<S: Store> Operation<S> for ReqRevise {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.reason.trim().is_empty() || self.source.trim().is_empty() {
            return Err(Error::invalid("revising needs a reason and a source"));
        }
        let mut node = node_of(repo, &self.id, NodeKind::Requirement)?;
        node.advance(RequirementState::Filed)?;
        if let NodeData::Requirement(data) = node.data_mut() {
            data.revisions.push(Revision::new(
                self.reason.clone(),
                self.source.clone(),
                now(),
            )?);
        }
        let why = format!("req revise {}", self.id);
        repo.transaction(&why, &self.source, |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed: vec!["revised".into()],
            missing,
            next,
            value: self.id,
        })
    }
}
