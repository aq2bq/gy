//! The write side: projecting an `Outcome` to output, resolving the scope from
//! `gy.toml`, and parsing the small closed sets the write commands take.
use gy_ledger::{ClosedBy, Closure, Error, NodeId, Outcome, Repository, Result, Store, config};
use serde::Serialize;
use std::fmt::{self, Display};
use std::path::Path;

/// What a write prints: the id, and what it changed, is missing, and can follow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Written {
    pub id: Option<String>,
    pub changed: Vec<String>,
    pub missing: Vec<String>,
    pub next: Vec<String>,
}
impl Written {
    pub fn of<T>(outcome: &Outcome<T>) -> Self {
        Self {
            id: outcome.id.as_ref().map(ToString::to_string),
            changed: outcome.changed.clone(),
            missing: outcome.missing.clone(),
            next: outcome.next.clone(),
        }
    }
}
impl Display for Written {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.id {
            writeln!(f, "{id}")?;
        }
        writeln!(f, "changed: {}", self.changed.join(", "))?;
        writeln!(f, "missing: {}", self.missing.join(", "))?;
        writeln!(f, "next: {}", self.next.join(", "))
    }
}

/// The scope a write uses: `--scope`, or the only one gy.toml lists.
pub fn scope(root: &Path, given: Option<&str>) -> Result<String> {
    let settings = config::read(root)?;
    if let Some(name) = given {
        if !settings.scopes.contains_key(name) {
            return Err(Error::invalid(format!("unknown scope {name}")));
        }
        return Ok(name.to_string());
    }
    match settings.scopes.len() {
        1 => Ok(settings.scopes.keys().next().expect("one scope").clone()),
        0 => Err(Error::invalid(
            "gy.toml has no scope; add one or pass --scope",
        )),
        _ => Err(Error::invalid("gy.toml has several scopes; pass --scope")),
    }
}

pub fn resolve_all<S: Store>(repository: &Repository<S>, texts: &[String]) -> Result<Vec<NodeId>> {
    texts.iter().map(|text| repository.resolve(text)).collect()
}

pub fn resolve_opt<S: Store>(
    repository: &Repository<S>,
    text: Option<&str>,
) -> Result<Option<NodeId>> {
    text.map(|text| repository.resolve(text)).transpose()
}

pub fn closed_by(text: &str) -> Result<ClosedBy> {
    match text {
        "fact" => Ok(ClosedBy::Fact),
        "external" => Ok(ClosedBy::External),
        _ => Err(Error::invalid(format!("unknown --by {text}"))),
    }
}

pub fn closure(text: &str) -> Result<Closure> {
    match text {
        "fact" => Ok(Closure::Fact),
        "decision" => Ok(Closure::Decision),
        "non-decision" => Ok(Closure::NonDecision),
        _ => Err(Error::invalid(format!("unknown --by {text}"))),
    }
}
