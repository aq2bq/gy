//! The named criterion date, kept as a free attribute (r-e9ca).
use super::{Node, NodeKind};
use crate::store::{Error, Result};
use chrono::NaiveDate;

pub(crate) fn due_date(text: &str) -> Option<NaiveDate> {
    let date = NaiveDate::parse_from_str(text, "%Y-%m-%d").ok()?;
    (date.format("%Y-%m-%d").to_string() == text).then_some(date)
}

/// Validate the final edit, so attributes can be set or removed together.
pub(crate) fn validate_wait(node: &Node) -> Result<()> {
    if node.kind() != NodeKind::Criterion {
        return Ok(());
    }
    let problem = match node.free("due") {
        Some(text) if due_date(text).is_none() => "due must be a valid date (YYYY-MM-DD)",
        None if node.free("responsible").is_some() || node.free("then").is_some() => {
            "a wait needs due; if the date is unknown, write the date when you will decide it"
        }
        _ => return Ok(()),
    };
    Err(Error::invalid(format!(
        "{problem}\nnext: gy edit {} --reason \"date the wait\" --set due=YYYY-MM-DD",
        node.id()
    )))
}
