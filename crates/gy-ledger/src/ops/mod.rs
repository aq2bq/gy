//! Operations: one intent = one command = one transaction (D-69, D-75).
pub mod advice;
pub mod config;
pub mod criterion_add;
pub mod criterion_satisfy;
pub mod decide;
pub mod edit;
pub mod link;
mod marks;
pub mod need_add;
pub mod need_close;
pub mod question_add;
pub mod question_close;
pub mod repository;
pub mod req_add;
pub mod req_approve;
pub mod req_cancel;
pub mod req_done;
pub mod req_revise;
pub mod scope_rename;
pub mod undo;

pub use criterion_add::CriterionAdd;
pub use criterion_satisfy::CriterionSatisfy;
pub use decide::Decide;
pub use edit::Edit;
pub use need_add::NeedAdd;
pub use need_close::NeedClose;
pub use question_add::QuestionAdd;
pub use question_close::QuestionClose;
pub use repository::Repository;
pub use req_add::ReqAdd;
pub use req_approve::ReqApprove;
pub use req_cancel::ReqCancel;
pub use req_done::ReqDone;
pub use req_revise::ReqRevise;
pub use scope_rename::ScopeRename;
pub use undo::Undo;

use crate::model::{Node, NodeId, NodeKind};
use crate::store::{Error, Result, Store};
use chrono::TimeZone;

/// Load a node of the expected kind, or an error naming what is wrong.
pub fn node_of<S: Store>(repo: &Repository<S>, id: &NodeId, kind: NodeKind) -> Result<Node> {
    let node = repo
        .get(id)?
        .ok_or_else(|| Error::invalid(format!("{id} does not exist")))?;
    if node.kind() != kind {
        return Err(Error::invalid(format!("{id} is not a {}", kind.name())));
    }
    Ok(node)
}

/// What an operation produced: the node it touched, what it changed, what is
/// missing, and what could follow (N-39).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome<T> {
    pub id: Option<NodeId>,
    pub changed: Vec<String>,
    pub missing: Vec<String>,
    pub next: Vec<String>,
    pub value: T,
}

/// What a node still lacks and what could follow (N-39), read from the graph
/// after the write so every command reports the same thing show does.
pub fn advice_for<S: Store>(
    repo: &Repository<S>,
    node: &Node,
) -> Result<(Vec<String>, Vec<String>)> {
    let all = repo.all()?;
    Ok((advice::missing(node, &all), advice::next(node, &all)))
}

/// One intent, run against a repository (D-69).
pub trait Operation<S: Store> {
    type Output;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>>;
}

/// The instant a write records: UTC, RFC 3339, seconds, `Z`. Every stored
/// instant — a node's `created`, a criterion's `satisfied_at`, a requirement's
/// approval / revision / completion / cancellation — takes this form (n-b6b6,
/// n-86cc).
pub fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// A stored `created` instant in the reader's own place, as `YYYY-MM-DD HH:MM`
/// for the human views (n-b6b6). `publish` and `--json` keep the UTC value; a
/// value that is not an RFC 3339 instant comes back unchanged.
pub fn local_time(created: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(created) {
        Ok(at) => at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        Err(_) => created.to_string(),
    }
}

/// The epoch second of the start of a local day named `YYYY-MM-DD`, in the
/// reader's own place, for `--since` (n-b6b6).
pub fn local_day_start(text: &str) -> Result<u64> {
    let shape = || {
        Error::invalid(format!(
            "unknown since {text}; expected a sequence or a date (YYYY-MM-DD)"
        ))
    };
    let date = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| shape())?;
    let midnight = date.and_hms_opt(0, 0, 0).ok_or_else(shape)?;
    match chrono::Local.from_local_datetime(&midnight) {
        chrono::LocalResult::Single(at) => Ok(at.timestamp().max(0) as u64),
        chrono::LocalResult::Ambiguous(first, _) => Ok(first.timestamp().max(0) as u64),
        chrono::LocalResult::None => Err(shape()),
    }
}
