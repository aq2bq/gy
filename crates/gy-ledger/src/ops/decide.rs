//! decide: create a decision, close its questions, and record lineage edges
//! with their marks, all in one transaction.
use super::{Operation, Outcome, Repository, advice_for, marks, today};
use crate::model::{
    Closure, DecisionScope, Link as ModelLink, Node, NodeData, NodeId, NodeKind, Relation,
};
use crate::store::{Error, Result, Store};

pub struct Decide {
    pub scope: String,
    pub title: String,
    pub decision_scope: DecisionScope,
    pub body: Option<String>,
    pub source: Option<String>,
    pub closes: Vec<NodeId>,
    pub relates: Vec<(Relation, NodeId, Option<String>)>,
}
impl<S: Store> Operation<S> for Decide {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.decision_scope.is_unrecorded() {
            return Err(Error::invalid(
                "the unrecorded marker is set only by migration",
            ));
        }
        let id = repo.next_id(NodeKind::Decision)?;
        let questions = close_questions(repo, &self.closes, &id)?;
        check_relates(repo, &self.relates)?;
        let node = self.build(&id)?;
        let source = self.source.clone().unwrap_or_else(|| "decide".to_string());
        let why = format!("decide {id}");
        repo.transaction(&why, &source, |repo| {
            repo.put(&node)?;
            for question in &questions {
                repo.put(question)?;
            }
            Ok(())
        })?;
        let (missing, next) = advice_for(repo, &node)?;
        Ok(Outcome {
            id: Some(id.clone()),
            changed: changed(&self.closes, &self.relates),
            missing,
            next,
            value: id,
        })
    }
}

impl Decide {
    fn build(&self, id: &NodeId) -> Result<Node> {
        let mut node = Node::decision(
            id.clone(),
            &self.scope,
            &today(),
            &self.title,
            self.decision_scope.clone(),
        )?;
        if let Some(body) = &self.body {
            node.set_body(body.clone());
        }
        for (relation, target, mark) in &self.relates {
            let edge = ModelLink::new(id.clone(), *relation, target.clone())?;
            node.link(edge.with_mark(mark.clone()));
        }
        Ok(node)
    }
}

fn changed(closes: &[NodeId], relates: &[(Relation, NodeId, Option<String>)]) -> Vec<String> {
    let mut changed = vec!["created".to_string()];
    changed.extend(closes.iter().map(|_| "closes".to_string()));
    changed.extend(
        relates
            .iter()
            .map(|(relation, _, _)| relation.name().to_string()),
    );
    changed
}

/// Close each question by this decision, adding the Closes edge on the
/// question side. The caller stages the returned nodes.
fn close_questions<S: Store>(
    repo: &Repository<S>,
    closes: &[NodeId],
    decision: &NodeId,
) -> Result<Vec<Node>> {
    let mut questions = Vec::new();
    for id in closes {
        let mut node = repo
            .get(id)?
            .ok_or_else(|| Error::invalid(format!("{id} does not exist")))?;
        if node.kind() != NodeKind::Question {
            return Err(Error::invalid(format!("{id} is not a question")));
        }
        match node.data_mut() {
            NodeData::Question(data) => {
                if data.closure.is_some() {
                    return Err(Error::invalid(format!("{id} is already closed")));
                }
                data.closure = Some(Closure::Decision);
                data.evidence = Some(decision.to_string());
            }
            _ => unreachable!(),
        }
        node.link(ModelLink::new(
            id.clone(),
            Relation::Closes,
            decision.clone(),
        )?);
        questions.push(node);
    }
    Ok(questions)
}

fn check_relates<S: Store>(
    repo: &Repository<S>,
    relates: &[(Relation, NodeId, Option<String>)],
) -> Result<()> {
    for (relation, id, mark) in relates {
        let lineage = matches!(
            relation,
            Relation::Narrows | Relation::Widens | Relation::Supersedes | Relation::Completes
        );
        if !lineage {
            return Err(Error::invalid(format!(
                "{} is not a lineage relation",
                relation.name()
            )));
        }
        let target = repo
            .get(id)?
            .ok_or_else(|| Error::invalid(format!("{id} does not exist")))?;
        if target.kind() != NodeKind::Decision {
            return Err(Error::invalid(format!("{id} is not a decision")));
        }
        marks::check(&target, mark.as_deref(), marks::required(*relation))?;
    }
    Ok(())
}
