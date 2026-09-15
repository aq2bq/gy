//! Node identity: the kind, the short-hash ID, old aliases, and the outward ref.
use crate::store::{Error, IdSource, Result};

/// The five node kinds. A gate is not a kind (D-68).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeKind {
    #[default]
    Need,
    Question,
    Decision,
    Requirement,
    Criterion,
}
impl NodeKind {
    pub const ALL: [Self; 5] = [
        Self::Need,
        Self::Question,
        Self::Decision,
        Self::Requirement,
        Self::Criterion,
    ];
    pub fn name(self) -> &'static str {
        ["need", "question", "decision", "requirement", "criterion"][self as usize]
    }
    pub fn prefix(self) -> &'static str {
        ["n", "q", "d", "r", "ac"][self as usize]
    }
}

/// A node identity: the kind and a short hash. There is no central counter
/// (D-74); a collision mints a longer hash, which N-38 implements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeId {
    kind: NodeKind,
    hash: String,
}
impl NodeId {
    pub fn from_hash(kind: NodeKind, hash: impl Into<String>) -> Result<Self> {
        let hash = hash.into();
        if hash.is_empty() {
            return Err(Error::invalid("a node hash must not be empty"));
        }
        Ok(Self { kind, hash })
    }
    pub fn mint(kind: NodeKind, ids: &mut dyn IdSource) -> Result<Self> {
        Ok(Self {
            kind,
            hash: ids.next_hash(kind.name())?,
        })
    }
    pub fn kind(&self) -> NodeKind {
        self.kind
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }
}
impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.kind.prefix(), self.hash)
    }
}

/// An old ID, kept so a renamed node stays reachable (D-74, N-41).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias(pub String);

/// An opaque outward reference. gy never reads its target (D-73).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref(pub String);
