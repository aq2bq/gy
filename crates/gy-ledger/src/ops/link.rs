//! link: add or remove one edge between two nodes. Closes is not allowed here
//! (question close and decide keep a closure and its edge together).
use super::{Operation, Outcome, Repository, marks};
use crate::model::{Link as ModelLink, Node, NodeId, NodeKind, Relation, decision_bound, find};
use crate::store::{Error, Result, Store};

#[derive(Clone)]
pub struct Link {
    pub from: NodeId,
    pub relation: Relation,
    pub to: NodeId,
    pub mark: Option<String>,
    pub remove: bool,
}
impl<S: Store> Operation<S> for Link {
    type Output = NodeId;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.relation == Relation::Closes {
            return Err(Error::invalid(
                "link cannot close; use question close or decide",
            ));
        }
        let mut from = repo
            .get(&self.from)?
            .ok_or_else(|| Error::invalid(format!("{} does not exist", self.from)))?;
        let to = repo
            .get(&self.to)?
            .ok_or_else(|| Error::invalid(format!("{} does not exist", self.to)))?;
        let existing = from
            .links()
            .iter()
            .position(|edge| !edge.reversed && edge.label == self.relation && edge.to == self.to);
        if self.remove {
            let Some(index) = existing else {
                return Err(Error::invalid("there is no such edge to remove"));
            };
            from.unlink(index);
            keeps_binding(repo, &from, &to)?;
        } else {
            marks::check(&to, self.mark.as_deref(), marks::required(self.relation))?;
            let edge = ModelLink::new(self.from.clone(), self.relation, self.to.clone())?;
            from.link(edge.with_mark(self.mark.clone()));
        }
        let why = format!("link {} {} {}", self.from, self.relation.name(), self.to);
        repo.transaction(&why, "link", |repo| repo.put(&from))?;
        Ok(Outcome {
            id: Some(self.from.clone()),
            changed: vec![if self.remove { "unlinked" } else { "linked" }.into()],
            missing: Vec::new(),
            next: Vec::new(),
            unresolved: Vec::new(),
            value: self.from,
        })
    }
}

/// Refuse a removal that would leave a decision with nothing that it is for
/// (ac-c6d2). The check reads the decision from either side of the edge: the
/// one the edge leaves, and the one it pointed at.
fn keeps_binding<S: Store>(repo: &Repository<S>, after: &Node, peer: &Node) -> Result<()> {
    let all = repo.all()?;
    let mut next = all.clone();
    if let Some(slot) = next.iter_mut().find(|node| node.id() == after.id()) {
        *slot = after.clone();
    }
    for id in [after.id(), peer.id()] {
        if id.kind() != NodeKind::Decision {
            continue;
        }
        let was = find(&all, id).is_some_and(|node| decision_bound(node, &all));
        let stays = find(&next, id).is_some_and(|node| decision_bound(node, &next));
        if was && !stays {
            return Err(Error::invalid(format!(
                "{id} would serve nothing: link {id} serves <N|R|AC> first"
            )));
        }
    }
    Ok(())
}
