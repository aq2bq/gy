//! criterion satisfy: record evidence that a criterion holds, or revoke it
//! while keeping the evidence.
use super::{Operation, Outcome, Repository, advice_for, now};
use crate::model::{Node, NodeData, NodeId, NodeKind};
use crate::store::{Error, Result, Store};

#[derive(Clone)]
pub struct CriterionSatisfy {
    pub id: NodeId,
    pub evidence: String,
    pub revoke: bool,
}
impl<S: Store> Operation<S> for CriterionSatisfy {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        let mut node = repo
            .get(&self.id)?
            .ok_or_else(|| Error::invalid(format!("{} does not exist", self.id)))?;
        if node.kind() != NodeKind::Criterion {
            return Err(Error::invalid(format!("{} is not a criterion", self.id)));
        }
        if self.evidence.trim().is_empty() {
            return Err(Error::invalid("satisfying a criterion needs evidence"));
        }
        update(&mut node, &self.id, &self.evidence, self.revoke)?;
        let why = format!("criterion satisfy {}", self.id);
        repo.transaction(&why, &self.evidence, |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed: vec!["satisfied".into()],
            missing,
            next,
            unresolved: Vec::new(),
            value: self.id,
        })
    }
}

fn update(node: &mut Node, id: &NodeId, evidence: &str, revoke: bool) -> Result<()> {
    match node.data_mut() {
        NodeData::Criterion(data) => {
            if revoke {
                data.satisfied = false;
                data.satisfied_at = None;
            } else if data.satisfied {
                return Err(Error::invalid(format!("{id} is already satisfied")));
            } else {
                data.satisfied = true;
                data.set_satisfied_at(now())?;
            }
            data.evidence = Some(evidence.to_string());
            Ok(())
        }
        _ => Err(Error::invalid(format!("{id} is not a criterion"))),
    }
}
