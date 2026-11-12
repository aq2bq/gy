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
        if self.remove {
            let index = removal(&from, self.relation, &self.to, self.mark.as_deref())?;
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

/// Select a stored passage, including one an edit has made unresolved. Older
/// copies of the same passage still remove one edge, as they always did.
fn removal(from: &Node, relation: Relation, to: &NodeId, mark: Option<&str>) -> Result<usize> {
    let candidates: Vec<_> = from
        .links()
        .iter()
        .enumerate()
        .filter(|(_, edge)| !edge.reversed && edge.label == relation && &edge.to == to)
        .collect();
    if relation == Relation::Narrows {
        if let Some(mark) = mark {
            return candidates
                .iter()
                .find(|(_, edge)| edge.mark.as_deref() == Some(mark))
                .map(|(index, _)| *index)
                .ok_or_else(|| Error::invalid("there is no such edge to remove"));
        }
        let marks: std::collections::BTreeSet<_> = candidates
            .iter()
            .map(|(_, edge)| edge.mark.as_deref())
            .collect();
        if marks.len() > 1 {
            return Err(Error::invalid(format!(
                "choose a passage with --mark to remove {} narrows {to}:\n{}",
                from.id(),
                marks
                    .into_iter()
                    .map(|mark| mark.unwrap_or("(unrecorded)"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
    }
    candidates
        .first()
        .map(|(index, _)| *index)
        .ok_or_else(|| Error::invalid("there is no such edge to remove"))
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
