//! In-progress requirements and the next evidence expected of them.
use crate::model::Node;
use crate::model::reference;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ProgressRow {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub state: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsible: Option<String>,
    /// The last writer as a reader sees them (n-d36d).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,
}
impl fmt::Display for ProgressRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match &self.reference {
            Some(reference) => format!("{} ({reference})", self.id),
            None => self.id.clone(),
        };
        writeln!(f, "{id} {} {}", self.state, self.title)?;
        if let Some(next_evidence) = &self.next_evidence {
            writeln!(f, "  next_evidence: {next_evidence}")?;
        }
        if let Some(responsible) = &self.responsible {
            writeln!(f, "  responsible: {responsible}")?;
        }
        if let Some(who) = &self.who {
            writeln!(f, "  who: {who}")?;
        }
        Ok(())
    }
}

pub(crate) fn progress(node: &Node, who: Option<String>) -> ProgressRow {
    ProgressRow {
        id: node.id().to_string(),
        reference: reference(node).map(str::to_string),
        state: node
            .state()
            .map(|state| state.name().to_string())
            .unwrap_or_default(),
        title: node.title().to_string(),
        next_evidence: node.free("next_evidence").cloned(),
        responsible: node.free("responsible").cloned(),
        who,
    }
}
