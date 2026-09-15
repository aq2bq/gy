//! publish: the record written as one file per node under a scope directory
//! (d-7c64, d-edb0). It is a development artifact to review later, not a
//! reading for the master.
mod nodes;

use crate::model::Node;
use crate::ops::repository::{Repository, Result, Store};

/// The files to write: one group per scope, each holding its node files.
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
) -> Result<Publication> {
    let all = repository.all()?;
    let mut scopes = Vec::new();
    for name in scope_names(&all, scope) {
        scopes.push(ScopeFiles {
            files: nodes::files(repository, &all, &name, since)?,
            name,
        });
    }
    Ok(Publication { scopes })
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
