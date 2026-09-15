//! question close: record the closure, its evidence, and, when a decision
//! closed it, the Closes edge.
use super::{Operation, Outcome, Repository};
use crate::model::{Closure, Link, NodeData, NodeId, NodeKind, Relation};
use crate::store::{Error, Result, Store};

pub struct QuestionClose {
    pub id: NodeId,
    pub by: Closure,
    pub evidence: String,
    pub decision: Option<NodeId>,
}
impl<S: Store> Operation<S> for QuestionClose {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        let mut node = repo
            .get(&self.id)?
            .ok_or_else(|| Error::invalid(format!("{} does not exist", self.id)))?;
        if node.kind() != NodeKind::Question {
            return Err(Error::invalid(format!("{} is not a question", self.id)));
        }
        if self.evidence.trim().is_empty() {
            return Err(Error::invalid("closing a question needs evidence"));
        } else if self.by == Closure::Decision && self.decision.is_none() {
            return Err(Error::invalid("closing by decision needs the decision"));
        }
        check_decision(repo, &self.decision)?;
        match node.data_mut() {
            NodeData::Question(data) => {
                if data.closure.is_some() {
                    return Err(Error::invalid(format!("{} is already closed", self.id)));
                }
                data.closure = Some(self.by);
                data.evidence = Some(self.evidence.clone());
            }
            _ => unreachable!(),
        }
        if let Some(decision) = &self.decision {
            node.link(Link::new(
                self.id.clone(),
                Relation::Closes,
                decision.clone(),
            )?);
        }
        let why = format!("question close {}", self.id);
        repo.transaction(&why, &self.evidence, |repo| repo.put(&node))?;
        Ok(Outcome {
            id: Some(self.id.clone()),
            changed: vec!["closed".into()],
            missing: Vec::new(),
            next: Vec::new(),
            value: self.id,
        })
    }
}

fn check_decision<S: Store>(repo: &Repository<S>, decision: &Option<NodeId>) -> Result<()> {
    if let Some(decision) = decision {
        match repo.get(decision)? {
            Some(node) if node.kind() == NodeKind::Decision => {}
            Some(_) => return Err(Error::invalid(format!("{decision} is not a decision"))),
            None => return Err(Error::invalid(format!("{decision} does not exist"))),
        }
    }
    Ok(())
}
