//! How a requirement moves and how a question closes.
use super::node::{Criterion, valid_created};
use crate::store::{Error, Result};
use serde::{Deserialize, Serialize};

/// The four requirement states (D-70). `Filed`: the request has been filed but
/// the design is not yet approved.
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

/// The approval that confirmed a requirement (D-70).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approval {
    pub design: String,
    pub heard_by: String,
    pub evidence: String,
    pub at: String,
}

/// One revision: an approved requirement sent back to filed, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    pub reason: String,
    pub source: String,
    pub at: String,
}

/// The completion of an approved requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Completion {
    pub evidence: String,
    pub at: String,
}

/// The cancellation of a requirement that was not done.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cancellation {
    pub reason: String,
    pub source: String,
    pub at: String,
}

/// The form every stored instant takes: the node's own `created` form (n-86cc).
fn valid_instant(text: &str) -> Result<()> {
    if valid_created(text) {
        Ok(())
    } else {
        Err(Error::invalid("the instant must be YYYY-MM-DDTHH:MM:SSZ"))
    }
}

impl Approval {
    /// An approval record at a UTC instant in the node's own form (n-86cc).
    pub fn new(design: String, heard_by: String, evidence: String, at: String) -> Result<Self> {
        valid_instant(&at)?;
        Ok(Self {
            design,
            heard_by,
            evidence,
            at,
        })
    }
}

impl Revision {
    /// A revision record at a UTC instant in the node's own form (n-86cc).
    pub fn new(reason: String, source: String, at: String) -> Result<Self> {
        valid_instant(&at)?;
        Ok(Self { reason, source, at })
    }
}

impl Completion {
    /// A completion record at a UTC instant in the node's own form (n-86cc).
    pub fn new(evidence: String, at: String) -> Result<Self> {
        valid_instant(&at)?;
        Ok(Self { evidence, at })
    }
}

impl Cancellation {
    /// A cancellation record at a UTC instant in the node's own form (n-86cc).
    pub fn new(reason: String, source: String, at: String) -> Result<Self> {
        valid_instant(&at)?;
        Ok(Self { reason, source, at })
    }
}

impl Criterion {
    /// Record the instant a criterion was satisfied, in the node's own form.
    pub fn set_satisfied_at(&mut self, at: String) -> Result<()> {
        valid_instant(&at)?;
        self.satisfied_at = Some(at);
        Ok(())
    }
}
