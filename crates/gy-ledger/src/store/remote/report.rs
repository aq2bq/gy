//! What one `gy remote sync` reports (n-6f47, n-ecbf): the sequences pulled and
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
    /// The writes put back on top of the remote's new lines (n-ecbf 2B).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rebased: Option<Range>,
    pub pushed: Option<Range>,
    /// How many writes were refused and left in `rejected.jsonl`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected: Option<usize>,
    /// The one-time branch protection, on the sync that pushed the first copy
    /// (ac-af33). Absent means nothing to say.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    /// True when an empty remote was filled from this copy (n-94bb 3B).
    #[serde(skip)]
    pub reuploaded: bool,
    #[serde(skip)]
    pub seq: u64,
    /// The greatest `Gy-Version` trailer pulled by this sync, when any
    /// (n-670a B). The binary compares it with its own release.
    #[serde(skip)]
    pub peer_version: Option<String>,
}
impl fmt::Display for Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pulled) = &self.pulled {
            pulled_line(f, pulled)?;
        }
        if let Some(pushed) = &self.pushed {
            range_line(f, "pushed", pushed)?;
        }
        if let Some(rebased) = &self.rebased {
            range_line(f, "rebased", rebased)?;
        }
        if let Some(rejected) = self.rejected {
            writeln!(f, "rejected: {rejected} writes")?;
        }
        if self.reuploaded {
            writeln!(f, "re-uploaded from this copy")?;
        }
        if self.is_up_to_date() {
            writeln!(f, "up to date: seq {}", self.seq)?;
        }
        if let Some(guard) = &self.guard {
            writeln!(f, "{guard}")?;
        }
        Ok(())
    }
}
impl Sync {
    fn is_up_to_date(&self) -> bool {
        self.pulled.is_none()
            && self.pushed.is_none()
            && self.rebased.is_none()
            && self.rejected.is_none()
            && !self.reuploaded
    }
}

/// `pulled: seq a → b (n writes by x, y)`.
fn pulled_line(f: &mut fmt::Formatter<'_>, pulled: &Pulled) -> fmt::Result {
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
    writeln!(f, ")")
}

/// `pushed` / `rebased`: `label: n writes (seq a → b)`. A reversed span (from >
/// to) is no span at all, so it reads as zero writes rather than underflowing
/// (n-8b91).
fn range_line(f: &mut fmt::Formatter<'_>, label: &str, range: &Range) -> fmt::Result {
    let writes = if range.to >= range.from {
        range.to - range.from + 1
    } else {
        0
    };
    writeln!(
        f,
        "{label}: {writes} writes (seq {} → {})",
        range.from, range.to
    )
}
