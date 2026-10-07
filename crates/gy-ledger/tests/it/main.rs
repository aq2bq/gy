//! The gy-ledger integration tests as one test executable (r-03c1, d-06cf):
//! the files under this directory are modules of this crate. Tests that
//! change process-wide environment variables keep their own files (see the
//! top-level `tests/*.rs` beside this one), so libtest does not run them in
//! the same process as one another.

#[path = "../common/mod.rs"]
mod common;

#[path = "../prop/derived.rs"]
mod derived;
#[path = "../prop/mod.rs"]
mod prop;
#[path = "../prop/rules.rs"]
mod rules;
#[path = "../prop/rules_e.rs"]
mod rules_e;

mod advice_sibling;
mod architecture;
mod as_of;
mod body_gap;
mod concurrency;
mod config;
mod criterion_add;
mod criterion_satisfy;
mod decide;
mod edit;
mod edit_marks;
mod ego;
mod file_store;
mod format;
mod format4;
mod freeze;
mod freeze_rebase;
mod gate_closed_need;
mod gate_closed_need_rebase;
mod gate_cycle;
mod gate_cycle_rebase;
mod gate_edges;
mod gate_edges_rebase;
mod gate_rebase;
mod gate_write;
mod handover;
mod ids;
mod link;
mod list;
mod model;
mod need_add;
mod need_close;
mod need_state;
mod next;
mod non_exhaustive;
mod now;
mod outcome;
mod prop_p1a;
mod prop_p1b;
mod prop_p3;
mod publish;
mod publish_scan;
mod question_add;
mod question_close;
mod repository;
mod req_add;
mod req_approve;
mod req_cancel;
mod req_done;
mod req_revise;
mod retraction;
mod retry;
mod rounds;
mod satisfy_rule;
mod scope_rename;
mod share;
mod show;
mod snapshot;
mod store;
mod sync;
mod sync_background;
mod sync_clone_race;
mod sync_format;
mod sync_format_guard;
mod sync_guard;
mod sync_lock;
mod sync_nested;
mod sync_notice;
mod sync_peer_version;
mod sync_rebase;
mod sync_rebase_landed;
mod sync_rebase_more;
mod sync_recovery;
mod sync_rejudge;
mod sync_report_range;
mod sync_timeout_waiter;
mod undo;
mod undo_op;
mod writer;
