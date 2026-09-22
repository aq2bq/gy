//! publish: the record written as a Markdown wiki a person reads on GitHub
//! (n-b6c9, d-90a1, d-50d1). One page per need and per decision, with the
//! nodes each reaches shown in place; the nodes no page reaches go to
//! `loose.md`. This need builds the pages, the front matter, the inlines, the
//! sharing and `loose.md`; the entry README comes with n-a391 (d-d82b).
mod home;
mod inline;
mod loose;
mod page;
mod wiki;

use super::show::show;
use crate::model::{Node, NodeKind};
use crate::ops::repository::{Repository, Result, Store};

/// The files to write: one group per scope, each holding that scope's pages.
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

/// The wiki for the named scope, or for every scope the ledger holds. `since`,
/// `writer` and `location` are accepted for the 1.0.1 API and not used
/// (d-3c54): the wiki always shows the record as it is now.
pub fn publish<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
    _since: Option<u64>,
    _writer: &str,
    _location: &str,
) -> Result<Publication> {
    let all = repository.all()?;
    let ids: Vec<String> = all.iter().map(|node| node.id().to_string()).collect();
    let wiki = wiki::Wiki::new(show(repository, &ids, true)?);
    let seq = repository
        .store()
        .history()
        .last()
        .map_or(0, |entry| entry.seq);
    let mut scopes = Vec::new();
    for name in scope_names(&all, scope) {
        let files = wiki.files(&name, seq);
        scopes.push(ScopeFiles { name, files });
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

/// Whether a kind gets a page of its own: the two vertices of the wiki.
pub(super) fn is_vertex(kind: NodeKind) -> bool {
    matches!(kind, NodeKind::Need | NodeKind::Decision)
}

/// Whether a kind is shown in full inside a vertex's page.
pub(super) fn is_inline(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Criterion | NodeKind::Requirement | NodeKind::Question
    )
}
