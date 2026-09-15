//! need close: record how a need closed, with evidence. Completion is the
//! need's own state (D-28).
use super::{Operation, Outcome, Repository, advice_for};
use crate::model::{Closed, ClosedBy, NodeData, NodeId, NodeKind};
use crate::store::{Error, Result, Store};

pub struct NeedClose {
    pub id: NodeId,
    pub by: ClosedBy,
    pub evidence: String,
}
impl<S: Store> Operation<S> for NeedClose {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        let mut node = repo
            .get(&self.id)?
            .ok_or_else(|| Error::invalid(format!("{} does not exist", self.id)))?;
        if node.kind() != NodeKind::Need {
            return Err(Error::invalid(format!("{} is not a need", self.id)));
        }
        if self.evidence.trim().is_empty() {
            return Err(Error::invalid("closing a need needs evidence"));
        }
        let source = self.evidence.clone();
        match node.data_mut() {
            NodeData::Need(data) => {
                if data.closed.is_some() {
                    return Err(Error::invalid(format!("{} is already closed", self.id)));
                }
                data.closed = Some(Closed {
                    by: self.by,
                    evidence: self.evidence,
                });
            }
            _ => unreachable!(),
        }
        let id = self.id;
        let why = format!("need close {id}");
        repo.transaction(&why, &source, |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(id.clone()),
            changed: vec!["closed".into()],
            missing,
            next,
            value: id,
        })
    }
}
