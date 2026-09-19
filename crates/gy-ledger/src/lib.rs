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
    ScopeRename, Undo, config, link, local_clock, local_day_start, local_time, retry,
};
pub use store::remote::{
    Join, Pulled, Range, Share, Sync, clear_rejected, join_check, join_notice, join_run, reconcile,
    record_timeout, record_timeout_after, rejected_notices, share_check, share_upload, sync,
};
pub use store::{
    Actor, Error, ErrorKind, FileStore, FormatVersion, HistoryEntry, IdSource, MemoryStore, Result,
    Store, SyncStatus, UndoneKind, file, format, location, log,
};
pub use views::{
    EdgeLine, Filter, Handover, Listing, LogRow, NeedState, NextRow, NodeRow, Now, ProgressRow,
    Publication, Ready, RequirementLine, Resume, Row, Shown, SyncRow, Waiting, Warning, handover,
    list, next, now, publish, requirement_in_progress, show,
};
