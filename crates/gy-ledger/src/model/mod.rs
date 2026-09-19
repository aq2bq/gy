//! The model layer: the five node types and their invariants, checked when a
//! node is built so the ledger is always valid (D-75, AC-53). It uses the
//! store layer only, for ID hashes and errors.
mod kind;
mod links;
mod node;
mod rule;
mod scope;
mod state;

pub use kind::{Alias, NodeId, NodeKind, Ref};
pub use links::{Edge, Link, Relation};
pub use node::{
    Attributes, Criterion, Decision, FreeAttributes, Need, Node, NodeData, Question, Requirement,
    bearer_count, free_attribute,
};
pub use scope::DecisionScope;
pub use state::{
    Approval, Cancellation, Closed, ClosedBy, Closure, Completion, RequirementState, Revision,
};

pub use rule::{Before, Change, admit, covered, filed_target};
