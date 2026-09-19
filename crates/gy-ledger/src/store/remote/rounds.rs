//! The serve log's failure throttle (n-08ae, ac-11c9): the same first error
//! line is shown once, then every tenth round; a recovery shows once. The
//! clock label is passed in, so this is pure and testable.
use std::fmt;

/// What consecutive failed rounds should print.
#[derive(Debug, Default)]
pub struct Rounds {
    key: Option<String>,
    since: Option<String>,
    failed: u64,
}

impl Rounds {
    pub fn new() -> Self {
        Self::default()
    }

    /// One failed round: the error's first line, gy's way out, and the clock
    /// label. Returns the lines to log (empty while the same failure repeats).
    pub fn failed(&mut self, first: &str, advice: Option<&str>, at: &str) -> Vec<String> {
        let first = first.to_string();
        if self.key.as_deref() != Some(first.as_str()) {
            self.key = Some(first.clone());
            self.since = Some(at.to_string());
            self.failed = 0;
        }
        self.failed += 1;
        if self.failed == 1 {
            let mut lines = vec![format!("{at} sync failed: {first}")];
            if let Some(advice) = advice {
                lines.push(format!("  → {advice}"));
            }
            return lines;
        }
        if self.failed % 10 == 1 {
            let since = self.since.as_deref().unwrap_or(at);
            return vec![format!("{at} still failing since {since} ({first})")];
        }
        Vec::new()
    }

    /// A successful round after failures: the one recovered line, even when
    /// the round itself had nothing to say.
    pub fn recovered(&mut self, at: &str) -> Vec<String> {
        if self.failed == 0 {
            return Vec::new();
        }
        let count = self.failed;
        self.key = None;
        self.since = None;
        self.failed = 0;
        vec![format!("{at} sync: recovered after {count} failed rounds")]
    }
}

impl fmt::Display for Rounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} failed rounds", self.failed)
    }
}
