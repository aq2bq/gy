//! scope rename: move every node of one scope to another, as one log change
//! (n-ff2b). The caller rewrites gy.toml after the transaction commits.
use super::{Operation, Outcome, Repository};
use crate::store::{Error, Result, Store};

pub struct ScopeRename {
    pub from: String,
    pub to: String,
}
impl<S: Store> Operation<S> for ScopeRename {
    type Output = usize;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        check(repo, &self.from, &self.to)?;
        let why = format!("scope rename {} {}", self.from, self.to);
        let nodes = repo.transaction(&why, "scope rename", |repo| {
            repo.rename_scope(&self.from, &self.to)
        })?;
        Ok(Outcome {
            id: None,
            changed: vec![format!(
                "scope: {} → {} ({nodes} nodes)",
                self.from, self.to
            )],
            missing: Vec::new(),
            next: Vec::new(),
            value: nodes,
        })
    }
}

/// The old name must be declared and the new one must be a new, usable name.
fn check<S: Store>(repo: &Repository<S>, from: &str, to: &str) -> Result<()> {
    if !repo.scopes().iter().any(|scope| scope == from) {
        return Err(Error::invalid(format!("unknown scope {from}")));
    }
    if repo.scopes().iter().any(|scope| scope == to) {
        return Err(Error::invalid(format!("scope {to} already exists")));
    }
    if to.is_empty() || !to.chars().all(usable) {
        return Err(Error::invalid(format!("{to} is not a usable scope name")));
    }
    Ok(())
}

fn usable(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}
