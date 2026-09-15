//! What the migration found and did, as lines for stdout and the report file.
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub before: BTreeMap<String, usize>,
    pub after: BTreeMap<String, usize>,
    pub aliases: usize,
    pub unrecorded: usize,
    pub waiting_on: usize,
    pub links: usize,
    pub dropped: Vec<String>,
    pub unmapped: Vec<String>,
    pub frozen: usize,
    pub publication: Option<String>,
    pub requirements: Vec<String>,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "before: {}", counts(&self.before))?;
        writeln!(f, "after: {}", counts(&self.after))?;
        writeln!(f, "aliases: {}", self.aliases)?;
        writeln!(f, "unrecorded decision scope: {}", self.unrecorded)?;
        writeln!(f, "waiting-on ids: {}", self.waiting_on)?;
        writeln!(f, "edges: {}", self.links)?;
        writeln!(f, "unmapped requirement status: {}", self.unmapped.len())?;
        for entry in &self.unmapped {
            writeln!(f, "  {entry}")?;
        }
        writeln!(f, "frozen records: {}", self.frozen)?;
        if let Some(publication) = &self.publication {
            writeln!(f, "publication: {publication}")?;
        }
        writeln!(f, "requirement states:")?;
        for entry in &self.requirements {
            writeln!(f, "  {entry}")?;
        }
        writeln!(f, "dropped attributes: {}", self.dropped.join(", "))
    }
}

fn counts(map: &BTreeMap<String, usize>) -> String {
    map.iter()
        .map(|(kind, count)| format!("{kind} {count}"))
        .collect::<Vec<_>>()
        .join(", ")
}
