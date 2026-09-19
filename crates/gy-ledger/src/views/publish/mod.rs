//! publish: the record written as one file per node under a scope directory,
//! with a scope index (d-7c64, d-edb0). It is a development artifact to review
//! later, not a reading for a person.
mod diagnostics;
mod history;
mod index;
mod nodes;

use crate::model::{Node, NodeKind};
use crate::ops::repository::{Repository, Result, Store};

/// The files to write: one group per scope, each holding its node files and
/// the scope index.
pub struct Publication {
    pub scopes: Vec<ScopeFiles>,
}

/// The files of one scope, with paths relative to `<out>/<scope>/`.
pub struct ScopeFiles {
    pub name: String,
    pub files: Vec<FileEntry>,
}

/// One file: a relative path and its text.
pub struct FileEntry {
    pub path: String,
    pub text: String,
}

/// The publication for the named scope, or for every scope the ledger holds.
pub fn publish<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
    since: Option<u64>,
    writer: &str,
    location: &str,
) -> Result<Publication> {
    let all = repository.all()?;
    let seq = log_seq(repository);
    let mut scopes = Vec::new();
    for name in scope_names(&all, scope) {
        let mut files = nodes::files(repository, &all, &name, since)?;
        files.push(index::index(
            repository, &all, &name, since, seq, writer, location,
        )?);
        scopes.push(ScopeFiles { name, files });
    }
    Ok(Publication { scopes })
}

/// The last write sequence, the publication's point in time.
fn log_seq<S: Store>(repository: &Repository<S>) -> u64 {
    repository
        .store()
        .history()
        .last()
        .map_or(0, |entry| entry.seq)
}

/// The scopes to publish, in a fixed order.
fn scope_names(all: &[Node], scope: Option<&str>) -> Vec<String> {
    match scope {
        Some(name) => vec![name.to_string()],
        None => {
            let mut names: Vec<String> = all.iter().map(|node| node.scope().to_string()).collect();
            names.sort();
            names.dedup();
            names
        }
    }
}

/// A node as another line refers to it: new ID, old alias, and title (d-edb0).
pub(super) fn reference(node: &Node) -> String {
    match node.aliases().first() {
        Some(alias) => format!("{} ({}) {}", node.id(), alias.0, node.title()),
        None => format!("{} {}", node.id(), node.title()),
    }
}

/// The directory a kind's files live in.
pub(super) fn plural(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Need => "needs",
        NodeKind::Question => "questions",
        NodeKind::Decision => "decisions",
        NodeKind::Requirement => "requirements",
        NodeKind::Criterion => "criteria",
    }
}

/// The relative path of a node's file.
pub(super) fn path(node: &Node) -> String {
    format!(
        "{}/{}-{}.md",
        plural(node.kind()),
        node.id(),
        safe_title(node.title())
    )
}

/// A title as a file name: only letters, digits, `-`, and `_` survive; the rest
/// becomes `-`, and the name stops at 40 characters.
fn safe_title(title: &str) -> String {
    let mut out = String::new();
    for ch in title.chars() {
        if out.chars().count() >= 40 {
            break;
        }
        if ch.is_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.to_string()
    }
}
