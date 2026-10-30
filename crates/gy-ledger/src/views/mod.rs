//! Derived views: projections that read through the model and ops only, never
//! the store directly (D-76). N-57 brings show, N-58 list, N-59 (2a) derive
//! and next; handover and publish follow.
mod due;
pub use due::{DatedWaiting, DueRow};
mod ego;
mod handover;
mod list;
mod next;
mod now;
mod progress;
mod publish;
mod retraction;
mod show;
mod sync_row;
mod writers;

pub use ego::{EGO_LIMIT, Ego, EgoEdge, EgoNode, ego};
pub use handover::{Handover, ProgressRow, Warning, handover, handover_on};
pub use list::{Filter, Listing, LogRow, Match, Row, list, matches, status_of};
pub use next::{NextRow, RequirementLine, next, next_on};
pub use now::{NodeRow, Now, Ready, Resume, Waiting, now};
pub use publish::{Publication, publish};
pub use retraction::{Narrowed, Retraction};
pub use show::{EdgeLine, Shown, WithdrawalLine, show};
pub use sync_row::{SyncRow, sync_row};

/// How a need, question, or requirement reads its open/closed state.
fn open_or_closed(closed: bool) -> &'static str {
    if closed { "closed" } else { "open" }
}
