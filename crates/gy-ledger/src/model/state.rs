//! How a requirement moves and how a question closes.
use crate::store::{Error, Result};
use serde::{Deserialize, Serialize};

/// The four requirement states (D-70). `Filed` is "起票済み": the request has
/// been filed but the design is not yet approved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RequirementState {
    #[default]
    Filed,
    Approved,
    Done,
    Cancelled,
}
impl RequirementState {
    pub fn name(self) -> &'static str {
        ["filed", "approved", "done", "cancelled"][self as usize]
    }
    /// Only filed→approved, approved→filed (revise), approved→done, and
    /// filed|approved→cancelled are valid (proposal-v2 §2).
    pub fn advance(self, to: Self) -> Result<Self> {
        let allowed = matches!(
            (self, to),
            (Self::Filed, Self::Approved)
                | (Self::Approved, Self::Filed)
                | (Self::Approved, Self::Done)
                | (Self::Filed, Self::Cancelled)
                | (Self::Approved, Self::Cancelled)
        );
        if allowed {
            Ok(to)
        } else {
            Err(Error::invalid(format!(
                "invalid requirement transition {} to {}",
                self.name(),
                to.name()
            )))
        }
    }
}

/// How a question closed (fact / decision / non-decision).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Closure {
    Fact,
    Decision,
    NonDecision,
}

/// How a need closed: a fact resolved it, or an external tracker did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClosedBy {
    Fact,
    External,
}

/// A need's closure and the evidence for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closed {
    pub by: ClosedBy,
    pub evidence: String,
}
