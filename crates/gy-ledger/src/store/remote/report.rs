//! What one `gy sync` reports (n-6f47, n-ecbf): the sequences pulled and
//! pushed, the one-time guard, and the copy's sequence after it.
use serde::Serialize;
use std::fmt;

/// A span of write sequences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Range {
    pub from: u64,
    pub to: u64,
}

/// The writes pulled, and the distinct actors they carry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pulled {
    pub from: u64,
    pub to: u64,
    pub writers: Vec<String>,
}

/// What one sync did. `seq` is the copy's sequence afterwards; it is not in
/// the JSON, whose absent sides are null.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Sync {
    pub pulled: Option<Pulled>,
    pub pushed: Option<Range>,
    /// The one-time branch protection, on the sync that pushed the first copy
    /// (ac-af33). Absent means nothing to say.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(skip)]
    pub seq: u64,
}
impl fmt::Display for Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pulled) = &self.pulled {
            write!(
                f,
                "pulled: seq {} → {} ({} writes",
                pulled.from,
                pulled.to,
                pulled.to - pulled.from
            )?;
            if !pulled.writers.is_empty() {
                write!(f, " by {}", pulled.writers.join(", "))?;
            }
            writeln!(f, ")")?;
        }
        if let Some(pushed) = &self.pushed {
            writeln!(
                f,
                "pushed: {} writes (seq {} → {})",
                pushed.to - pushed.from + 1,
                pushed.from,
                pushed.to
            )?;
        }
        if self.pulled.is_none() && self.pushed.is_none() {
            writeln!(f, "up to date: seq {}", self.seq)?;
        }
        if let Some(guard) = &self.guard {
            writeln!(f, "{guard}")?;
        }
        Ok(())
    }
}
