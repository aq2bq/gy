//! Relationships: the closed set of edge labels, and a link that carries both
//! directions.
use super::NodeId;

/// The closed set of canonical relationships (D-76). Each has an inverse name,
/// so a link derives the other direction rather than being handed it. Whether
/// a kind may carry a relation (a question closing a decision, for example) is
/// checked in N-37.
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
    pub fn name(self) -> &'static str {
        Self::PAIRS[self as usize].0
    }
    pub fn inverse(self) -> &'static str {
        Self::PAIRS[self as usize].1
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
    pub fn new(from: NodeId, relation: Relation, to: NodeId) -> Self {
        Self {
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
        }
    }
    pub fn forward(&self) -> &Edge {
        &self.forward
    }
    pub fn reverse(&self) -> &Edge {
        &self.reverse
    }
}
