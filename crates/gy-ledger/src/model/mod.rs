//! The model layer: the five node types and their invariants, checked when a
//! node is built so the ledger is always valid (D-75, AC-53). It uses the
//! store layer only, for ID hashes and errors.
mod derive;
mod due;
mod edges;
mod kind;
pub(crate) use derive::criterion_wait;
pub(crate) use due::{due_date, validate_wait};
mod links;
mod node;
mod rule;
mod scope;
mod shape;
mod state;

pub use derive::{
    CriterionState, NeedState, bearer_count, covered, criterion_satisfied, criterion_state,
    criterion_unsatisfied, decision_bound, edges, filed_target, find, need_closed, need_state,
    question_open, ready, reference, requirement_in_progress, unmet_orphaned, unwaited,
    withdrawn_by,
};
pub use kind::{Alias, NodeId, NodeKind, Ref};
pub use links::{Edge, Link, Relation};
pub use node::{
    Attributes, Criterion, Decision, FreeAttributes, Need, Node, NodeData, Question, Requirement,
    free_attribute,
};
pub use scope::DecisionScope;
pub use state::{
    Approval, Cancellation, Closed, ClosedBy, Closure, Completion, RequirementState, Revision,
};

pub use rule::{Before, Change, admit};
