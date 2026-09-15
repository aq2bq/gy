//! Relationships: the closed set of edge labels, and a link that carries both
//! directions.
use super::{NodeId, NodeKind};
use crate::store::{Error, Result};

/// The closed set of canonical relationships (D-76). Each has an inverse name,
/// so a link derives the other direction rather than being handed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Closes,
    Narrows,
    Widens,
    Supersedes,
    Completes,
    Targets,
    SpawnedBy,
    FiledAs,
    DependsOn,
    ReliesOn,
    Raised,
}
impl Relation {
    const PAIRS: [(&'static str, &'static str); 11] = [
        ("closes", "closed-by"),
        ("narrows", "narrowed-by"),
        ("widens", "widened-by"),
        ("supersedes", "superseded-by"),
        ("completes", "completed-by"),
        ("targets", "targeted-by"),
        ("spawned-by", "spawns"),
        ("filed-as", "files"),
        ("depends-on", "depended-on-by"),
        ("relies-on", "relied-on-by"),
        ("raised", "raised-by"),
    ];
    /// The (from kind, to kind) pairs each relation allows. One table, in the
    /// model, so an edge that breaks a kind pairing cannot be built (D-75).
    const ALLOWED: &'static [(Self, NodeKind, NodeKind)] = &[
        (Self::Closes, NodeKind::Question, NodeKind::Decision),
        (Self::Narrows, NodeKind::Decision, NodeKind::Decision),
        (Self::Widens, NodeKind::Decision, NodeKind::Decision),
        (Self::Supersedes, NodeKind::Decision, NodeKind::Decision),
        (Self::Completes, NodeKind::Decision, NodeKind::Decision),
        (Self::Targets, NodeKind::Need, NodeKind::Criterion),
        (Self::Targets, NodeKind::Requirement, NodeKind::Criterion),
        (Self::SpawnedBy, NodeKind::Need, NodeKind::Decision),
        (Self::FiledAs, NodeKind::Need, NodeKind::Requirement),
        (Self::DependsOn, NodeKind::Need, NodeKind::Need),
        (Self::ReliesOn, NodeKind::Requirement, NodeKind::Decision),
        (Self::Raised, NodeKind::Requirement, NodeKind::Question),
    ];
    pub fn name(self) -> &'static str {
        Self::PAIRS[self as usize].0
    }
    pub fn inverse(self) -> &'static str {
        Self::PAIRS[self as usize].1
    }
    fn allows(self, from: NodeKind, to: NodeKind) -> bool {
        Self::ALLOWED
            .iter()
            .any(|(relation, source, target)| *relation == self && *source == from && *target == to)
    }
}

/// A directed edge. `label` is the relation and `reversed` marks the inverse
/// side, so the name is derived from the relation and never stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: NodeId,
    pub label: Relation,
    pub reversed: bool,
    pub to: NodeId,
}
impl Edge {
    pub fn name(&self) -> &'static str {
        if self.reversed {
            self.label.inverse()
        } else {
            self.label.name()
        }
    }
}

/// A relationship carrying both directions, so a one-sided edge cannot exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    forward: Edge,
    reverse: Edge,
}
impl Link {
    /// Build both directions of a relationship, rejecting a pair whose kinds
    /// the relation does not allow (D-75).
    pub fn new(from: NodeId, relation: Relation, to: NodeId) -> Result<Self> {
        if !relation.allows(from.kind(), to.kind()) {
            return Err(Error::invalid(format!(
                "{} may not {} {}",
                from.kind().name(),
                relation.name(),
                to.kind().name()
            )));
        }
        Ok(Self {
            forward: Edge {
                from: from.clone(),
                label: relation,
                reversed: false,
                to: to.clone(),
            },
            reverse: Edge {
                from: to,
                label: relation,
                reversed: true,
                to: from,
            },
        })
    }
    pub fn forward(&self) -> &Edge {
        &self.forward
    }
    pub fn reverse(&self) -> &Edge {
        &self.reverse
    }
}
