//! Derived views: projections that read through the model and ops only, never
//! the store directly (D-76). N-57 brings show, N-58 list, N-59 (2a) derive
//! and next; handover and publish follow.
mod ego;
mod handover;
mod list;
mod next;
mod now;
mod publish;
mod retraction;
mod show;
mod sync_row;
mod writers;

pub use ego::{EGO_LIMIT, Ego, EgoEdge, EgoNode, ego};
pub use handover::{Handover, ProgressRow, Warning, handover};
pub use list::{Filter, Listing, LogRow, Match, Row, list, matches};
pub use next::{NextRow, RequirementLine, next};
pub use now::{NodeRow, Now, Ready, Resume, Waiting, now};
pub use publish::{Publication, publish};
pub use retraction::{Narrowed, Retraction};
pub use show::{EdgeLine, Shown, show};
pub use sync_row::SyncRow;

/// How a need, question, or requirement reads its open/closed state.
fn open_or_closed(closed: bool) -> &'static str {
    if closed { "closed" } else { "open" }
}
