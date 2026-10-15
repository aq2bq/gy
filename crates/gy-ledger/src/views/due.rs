//! Dated, unmet criteria as the resume views read them (r-e9ca).
use crate::model::{Node, criterion_wait};
use chrono::NaiveDate;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DueRow {
    pub id: String,
    pub due: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsible: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub then: Option<String>,
    pub status: &'static str,
}
impl DueRow {
    pub(crate) fn of(node: &Node, today: NaiveDate, scope: Option<&str>) -> Option<Self> {
        if scope.is_some_and(|scope| node.scope() != scope) {
            return None;
        }
        let (due, status) = criterion_wait(node, today)?;
        Some(Self {
            id: node.id().to_string(),
            due: due.to_string(),
            title: node.title().to_string(),
            responsible: node.free("responsible").cloned(),
            then: node.free("then").cloned(),
            status,
        })
    }
}
impl fmt::Display for DueRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} due {} {}", self.id, self.due, self.title)?;
        for (key, value) in [("responsible", &self.responsible), ("then", &self.then)] {
            if let Some(value) = value {
                writeln!(f, "  {key}: {value}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DatedWaiting {
    pub count: usize,
    pub earliest: String,
}

pub(crate) fn dated(
    all: &[Node],
    scope: Option<&str>,
    today: NaiveDate,
) -> (Vec<DueRow>, Option<DatedWaiting>) {
    let (mut due, future): (Vec<_>, Vec<_>) = all
        .iter()
        .filter_map(|node| DueRow::of(node, today, scope))
        .partition(|row| row.status == "due");
    due.sort_by(|a, b| a.due.cmp(&b.due).then(a.id.cmp(&b.id)));
    let waiting = future
        .iter()
        .map(|row| &row.due)
        .min()
        .map(|earliest| DatedWaiting {
            count: future.len(),
            earliest: earliest.clone(),
        });
    (due, waiting)
}
