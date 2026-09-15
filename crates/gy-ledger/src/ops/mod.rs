//! Operations: one intent = one command = one transaction (D-69, D-75).
pub mod config;
pub mod criterion_add;
pub mod criterion_satisfy;
pub mod need_add;
pub mod need_close;
pub mod question_add;
pub mod question_close;
pub mod repository;

pub use criterion_add::CriterionAdd;
pub use criterion_satisfy::CriterionSatisfy;
pub use need_add::NeedAdd;
pub use need_close::NeedClose;
pub use question_add::QuestionAdd;
pub use question_close::QuestionClose;
pub use repository::Repository;

use crate::model::NodeId;
use crate::store::{Result, Store};

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

/// One intent, run against a repository (D-69).
pub trait Operation<S: Store> {
    type Output;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>>;
}

/// Today's date, `YYYY-MM-DD`.
pub fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}
