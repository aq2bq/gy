//! link: add or remove one edge between two nodes. Closes is not allowed here
//! (question close and decide keep a closure and its edge together).
use super::{Operation, Outcome, Repository, marks};
use crate::model::{Link as ModelLink, NodeId, Relation};
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
        } else {
            if existing.is_some() {
                return Err(Error::invalid("that edge already exists"));
            }
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
            value: self.from,
        })
    }
}
