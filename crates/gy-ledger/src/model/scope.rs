//! A decision's applicability conditions.
use crate::store::{Error, Result};
use serde::{Deserialize, Serialize};

/// An empty scope needs the unrecorded marker, which only the migration sets
/// (proposal-v2 §7, N-41).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionScope {
    text: String,
    unrecorded: bool,
}
impl DecisionScope {
    pub fn recorded(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(Error::invalid(
                "a decision needs applicability conditions or the migration marker",
            ));
        }
        Ok(Self {
            text,
            unrecorded: false,
        })
    }
    /// The one path for an old decision whose scope was never recorded.
    pub fn migration_unrecorded() -> Self {
        Self {
            text: String::new(),
            unrecorded: true,
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn is_unrecorded(&self) -> bool {
        self.unrecorded
    }
}
