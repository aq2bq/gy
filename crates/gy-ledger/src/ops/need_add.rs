//! need add: a need targeting existing criteria, optionally spawned by a
//! decision. It writes both sides from the need's own links.
use super::{Operation, Outcome, Repository, advice_for, now};
use crate::model::{Link, Node, NodeId, NodeKind, Relation};
use crate::store::{Error, Result, Store};

#[derive(Clone)]
pub struct NeedAdd {
    pub scope: String,
    pub title: String,
    pub targets: Vec<NodeId>,
    pub spawned_by: Option<NodeId>,
    pub body: Option<String>,
}
impl<S: Store> Operation<S> for NeedAdd {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.targets.is_empty() {
            return Err(Error::invalid("a need needs at least one target"));
        }
        check_targets(repo, &self.targets)?;
        check_spawn(repo, &self.spawned_by)?;
        let id = repo.next_id(NodeKind::Need)?;
        let mut node = Node::need(id.clone(), &self.scope, &now(), &self.title)?;
        for target in &self.targets {
            node.link(Link::new(id.clone(), Relation::Targets, target.clone())?);
        }
        if let Some(decision) = &self.spawned_by {
            node.link(Link::new(
                id.clone(),
                Relation::SpawnedBy,
                decision.clone(),
            )?);
        }
        if let Some(body) = &self.body {
            node.set_body(body.clone());
        }
        let why = format!("need add {id}");
        repo.transaction(&why, "need add", |repo| repo.put(&node))?;
        let (missing, next) = advice_for(repo, &node)?;
        let mut changed = vec!["created".to_string(), "targets".to_string()];
        if self.spawned_by.is_some() {
            changed.push("spawned-by".to_string());
        }
        Ok(Outcome {
            id: Some(id.clone()),
            changed,
            missing,
            next,
            value: id,
        })
    }
}

fn check_targets<S: Store>(repo: &Repository<S>, targets: &[NodeId]) -> Result<()> {
    for target in targets {
        match repo.get(target)? {
            Some(node) if node.kind() == NodeKind::Criterion => {}
            Some(_) => return Err(Error::invalid(format!("{target} is not a criterion"))),
            None => return Err(Error::invalid(format!("{target} does not exist"))),
        }
    }
    Ok(())
}

fn check_spawn<S: Store>(repo: &Repository<S>, spawned_by: &Option<NodeId>) -> Result<()> {
    let Some(decision) = spawned_by else {
        return Ok(());
    };
    match repo.get(decision)? {
        Some(node) if node.kind() == NodeKind::Decision => Ok(()),
        Some(_) => Err(Error::invalid(format!("{decision} is not a decision"))),
        None => Err(Error::invalid(format!("{decision} does not exist"))),
    }
}
