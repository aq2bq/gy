//! next: the needs ready to work, with how many of their criteria are
//! satisfied and which requirements they became. The agent picks one (D-62).
use super::derive::{edges, find, ready, reference};
use crate::model::{Node, NodeData, NodeKind, Relation};
use crate::ops::repository::{Repository, Result, Store};
use serde::Serialize;
use std::fmt;

/// A filed requirement as next shows it: id, ref, state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementLine {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub state: String,
}
impl fmt::Display for RequirementLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.reference {
            Some(reference) => write!(f, "{} ({reference}) {}", self.id, self.state),
            None => write!(f, "{} {}", self.id, self.state),
        }
    }
}

/// One ready need: id, title, scope, satisfied / total criteria, and its filed
/// requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NextRow {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub satisfied: usize,
    pub targets: usize,
    pub requirements: Vec<RequirementLine>,
}
impl fmt::Display for NextRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {} ({}) targets {}/{}",
            self.id, self.title, self.scope, self.satisfied, self.targets
        )?;
        for requirement in &self.requirements {
            writeln!(f, "  {requirement}")?;
        }
        Ok(())
    }
}

/// The ready needs, ordered by created then id (no priority; D-62).
pub fn next<S: Store>(repo: &Repository<S>, scope: Option<&str>) -> Result<Vec<NextRow>> {
    Ok(ready_rows(&repo.all()?, scope))
}

/// The ready rows, so `now` reports the same judgement and order as `next`.
pub(crate) fn ready_rows(all: &[Node], scope: Option<&str>) -> Vec<NextRow> {
    let mut needs: Vec<&Node> = all
        .iter()
        .filter(|node| node.kind() == NodeKind::Need)
        .filter(|node| scope.is_none_or(|scope| node.scope() == scope))
        .filter(|node| ready(node, all))
        .collect();
    needs.sort_by(|a, b| {
        a.created()
            .cmp(b.created())
            .then_with(|| a.id().to_string().cmp(&b.id().to_string()))
    });
    needs.iter().map(|need| row(need, all)).collect()
}

fn row(need: &Node, all: &[Node]) -> NextRow {
    let targets = edges(need, Relation::Targets);
    let satisfied = targets
        .iter()
        .filter(|id| {
            find(all, id).is_some_and(
                |node| matches!(node.data(), NodeData::Criterion(data) if data.satisfied),
            )
        })
        .count();
    let requirements = edges(need, Relation::FiledAs)
        .iter()
        .filter_map(|id| find(all, id))
        .map(|node| RequirementLine {
            id: node.id().to_string(),
            reference: reference(node),
            state: node
                .state()
                .map(|state| state.name().to_string())
                .unwrap_or_default(),
        })
        .collect();
    NextRow {
        id: need.id().to_string(),
        title: need.title().to_string(),
        scope: need.scope().to_string(),
        satisfied,
        targets: targets.len(),
        requirements,
    }
}
