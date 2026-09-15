//! req add: a requirement filed against one or more needs, with optional
//! decisions it relies on, criteria it targets, and one outward reference.
use super::{Operation, Outcome, Repository, advice_for, node_of, today};
use crate::model::{
    Link as ModelLink, Node, NodeData, NodeId, NodeKind, Ref, Relation, RequirementState,
};
use crate::store::{Error, Result, Store};

pub struct ReqAdd {
    pub scope: String,
    pub title: String,
    pub needs: Vec<NodeId>,
    pub relies_on: Vec<NodeId>,
    pub targets: Vec<NodeId>,
    pub reference: Option<Ref>,
}
impl<S: Store> Operation<S> for ReqAdd {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.needs.is_empty() {
            return Err(Error::invalid("a requirement needs at least one need"));
        }
        check_needs(repo, &self.needs)?;
        kind_check(repo, &self.relies_on, NodeKind::Decision)?;
        kind_check(repo, &self.targets, NodeKind::Criterion)?;
        check_reference(repo, self.reference.as_ref())?;

        let id = repo.next_id(NodeKind::Requirement)?;
        let node = self.build(&id)?;
        let needs = filed_as_edges(repo, &self.needs, &id)?;
        let source = self
            .reference
            .as_ref()
            .map_or_else(|| "req add".to_string(), |r| r.0.clone());
        let why = format!("req add {id}");
        repo.transaction(&why, &source, |repo| {
            repo.put(&node)?;
            for need in &needs {
                repo.put(need)?;
            }
            Ok(())
        })?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(id.clone()),
            changed: vec!["created".into(), "needs".into()],
            missing,
            next,
            value: id,
        })
    }
}

impl ReqAdd {
    fn build(&self, id: &NodeId) -> Result<Node> {
        let mut node = Node::requirement(
            id.clone(),
            &self.scope,
            &today(),
            &self.title,
            RequirementState::Filed,
        )?;
        if let NodeData::Requirement(data) = node.data_mut() {
            data.reference = self.reference.clone();
        }
        for decision in &self.relies_on {
            node.link(ModelLink::new(
                id.clone(),
                Relation::ReliesOn,
                decision.clone(),
            )?);
        }
        for criterion in &self.targets {
            node.link(ModelLink::new(
                id.clone(),
                Relation::Targets,
                criterion.clone(),
            )?);
        }
        Ok(node)
    }
}

fn check_needs<S: Store>(repo: &Repository<S>, needs: &[NodeId]) -> Result<()> {
    for id in needs {
        let need = node_of(repo, id, NodeKind::Need)?;
        if let NodeData::Need(data) = need.data() {
            if data.closed.is_some() {
                return Err(Error::invalid(format!("{id} is closed")));
            }
        }
    }
    Ok(())
}

fn kind_check<S: Store>(repo: &Repository<S>, ids: &[NodeId], kind: NodeKind) -> Result<()> {
    for id in ids {
        node_of(repo, id, kind)?;
    }
    Ok(())
}

fn check_reference<S: Store>(repo: &Repository<S>, reference: Option<&Ref>) -> Result<()> {
    let Some(reference) = reference else {
        return Ok(());
    };
    for node in repo.all()? {
        let NodeData::Requirement(data) = node.data() else {
            continue;
        };
        let same = data.reference.as_ref().is_some_and(|r| r.0 == reference.0);
        let open = matches!(
            data.state,
            RequirementState::Filed | RequirementState::Approved
        );
        if same && open {
            return Err(Error::invalid(format!(
                "{} already has reference {}",
                node.id(),
                reference.0
            )));
        }
    }
    Ok(())
}

fn filed_as_edges<S: Store>(
    repo: &Repository<S>,
    needs: &[NodeId],
    requirement: &NodeId,
) -> Result<Vec<Node>> {
    let mut out = Vec::new();
    for id in needs {
        let mut need = node_of(repo, id, NodeKind::Need)?;
        need.link(ModelLink::new(
            id.clone(),
            Relation::FiledAs,
            requirement.clone(),
        )?);
        out.push(need);
    }
    Ok(out)
}
