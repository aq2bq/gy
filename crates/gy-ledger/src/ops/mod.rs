//! Operations: one intent = one command = one transaction (D-69, D-75).
pub mod config;
pub mod criterion_add;
pub mod criterion_satisfy;
pub mod decide;
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

pub use criterion_add::CriterionAdd;
pub use criterion_satisfy::CriterionSatisfy;
pub use decide::Decide;
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

use crate::model::{Node, NodeId, NodeKind};
use crate::store::{Error, Result, Store};

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

/// The outcome of a state transition: the same id, one changed label.
pub fn outcome(id: &NodeId, changed: &str) -> Outcome<NodeId> {
    Outcome {
        id: Some(id.clone()),
        changed: vec![changed.to_string()],
        missing: Vec::new(),
        next: Vec::new(),
        value: id.clone(),
    }
}

/// One intent, run against a repository (D-69).
pub trait Operation<S: Store> {
    type Output;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>>;
}

/// Today's date, `YYYY-MM-DD`.
pub fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}
