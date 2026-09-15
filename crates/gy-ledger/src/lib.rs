//! gy's new core, split into four layers with dependencies pointing upward
//! only (D-76): `store` (transactions, ID generation, history, format
//! version), then `model` (the five node types and their invariants), then
//! `ops` (operations), then `views` (derived projections).
//!
//! Only `model::free_attribute` reads a node attribute by a string key; every
//! other field is typed.
mod model;
mod ops;
mod store;
mod views;

pub use model::{
    Alias, Approval, Attributes, Cancellation, Closed, ClosedBy, Closure, Completion, Criterion,
    Decision, DecisionScope, Edge, FreeAttributes, Link, Need, Node, NodeData, NodeId, NodeKind,
    Question, Ref, Relation, Requirement, RequirementState, Revision, bearer_count, free_attribute,
};
pub use ops::{
    CriterionAdd, CriterionSatisfy, Decide, Edit, NeedAdd, NeedClose, Operation, Outcome,
    QuestionAdd, QuestionClose, Repository, ReqAdd, ReqApprove, ReqCancel, ReqDone, ReqRevise,
    Undo, config, link,
};
pub use store::{
    Actor, Error, FileStore, FormatVersion, HistoryEntry, IdSource, MemoryStore, Result, Store,
    file, format, location, log,
};
pub use views::{
    EdgeLine, Filter, Handover, Listing, LogRow, NeedState, NextRow, ProgressRow, RequirementLine,
    Row, Shown, Warning, describe, handover, list, next, publish, requirement_in_progress, show,
};
