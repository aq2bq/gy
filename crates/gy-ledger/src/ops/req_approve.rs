//! req approve: record the design, who heard it, and the evidence, moving a
//! filed requirement to approved (D-70).
use super::{Operation, Outcome, Repository, advice_for, node_of, now};
use crate::model::{Approval, NodeData, NodeId, NodeKind, RequirementState};
use crate::store::{Error, Result, Store};

#[derive(Clone)]
pub struct ReqApprove {
    pub id: NodeId,
    pub design: String,
    pub heard_by: String,
    pub evidence: String,
}
impl<S: Store> Operation<S> for ReqApprove {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.design.trim().is_empty()
            || self.heard_by.trim().is_empty()
            || self.evidence.trim().is_empty()
        {
            return Err(Error::invalid(
                "approving needs design, heard_by, and evidence",
            ));
        }
        let mut node = node_of(repo, &self.id, NodeKind::Requirement)?;
        node.advance(RequirementState::Approved)?;
        if let NodeData::Requirement(data) = node.data_mut() {
            data.approval = Some(Approval::new(
                self.design.clone(),
                self.heard_by.clone(),
                self.evidence.clone(),
                now(),
            )?);
        }
        let why = format!("req approve {}", self.id);
        repo.transaction(&why, &self.design, |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed: vec!["approved".into()],
            missing,
            next,
            value: self.id,
        })
    }
}
