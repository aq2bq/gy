//! The write side: projecting an `Outcome` to output, resolving the scope from
//! `gy.toml`, and parsing the small closed sets the write commands take.
use gy_ledger::{
    ClosedBy, Closure, Error, NodeData, NodeId, Outcome, Relation, Repository, Result, Store,
    config,
};
use serde::Serialize;
use std::fmt::{self, Display};
use std::path::Path;

/// What a write prints: the id, and what it changed, is missing, and can follow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Written {
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub changed: Vec<String>,
    pub missing: Vec<String>,
    pub next: Vec<String>,
}
impl Written {
    pub fn of<T>(outcome: &Outcome<T>) -> Self {
        Self {
            id: outcome.id.as_ref().map(ToString::to_string),
            reference: None,
            changed: outcome.changed.clone(),
            missing: outcome.missing.clone(),
            next: outcome.next.clone(),
        }
    }
    /// Put a requirement's outward reference beside its id (proposal-v3 12).
    pub fn reference(mut self, reference: Option<String>) -> Self {
        self.reference = reference;
        self
    }
}
impl Display for Written {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.id {
            match &self.reference {
                Some(reference) => writeln!(f, "{id} ({reference})")?,
                None => writeln!(f, "{id}")?,
            }
        }
        writeln!(f, "changed: {}", self.changed.join(", "))?;
        writeln!(f, "missing: {}", self.missing.join(", "))?;
        writeln!(f, "next: {}", self.next.join(", "))
    }
}

/// A requirement's outward reference, for the id line.
pub fn requirement_reference<S: Store>(
    repository: &Repository<S>,
    id: &NodeId,
) -> Result<Option<String>> {
    let node = repository
        .get(id)?
        .ok_or_else(|| Error::invalid(format!("{id} does not exist")))?;
    Ok(match node.data() {
        NodeData::Requirement(data) => data.reference.as_ref().map(|ref_| ref_.0.clone()),
        _ => None,
    })
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

/// The canonical name of any relation a `link` may name.
pub fn relation(text: &str) -> Result<Relation> {
    [
        Relation::Closes,
        Relation::Narrows,
        Relation::Widens,
        Relation::Supersedes,
        Relation::Completes,
        Relation::Targets,
        Relation::SpawnedBy,
        Relation::FiledAs,
        Relation::DependsOn,
        Relation::ReliesOn,
        Relation::Raised,
    ]
    .into_iter()
    .find(|relation| relation.name() == text)
    .ok_or_else(|| Error::invalid(format!("unknown relation {text}")))
}

/// `key=value` pairs, as `--set` and `--append` take them.
pub fn pairs(values: &[String]) -> Result<Vec<(String, String)>> {
    values
        .iter()
        .map(|value| {
            value
                .split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .ok_or_else(|| Error::invalid(format!("expected key=value, got {value}")))
        })
        .collect()
}

/// A body read from a file, when one is named.
pub fn body_file(path: Option<&str>) -> Result<Option<String>> {
    path.map(|path| {
        std::fs::read_to_string(path).map_err(|error| Error::invalid(format!("{path}: {error}")))
    })
    .transpose()
}
