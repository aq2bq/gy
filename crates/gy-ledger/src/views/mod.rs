//! Derived views: projections that read through the model and ops only, never
//! the store directly (D-76). N-57 brings show, N-58 list, N-59 (2a) derive
//! and next; handover and publish follow.
mod derive;
mod handover;
mod list;
mod next;
mod publish;
mod show;

pub use derive::{NeedState, requirement_in_progress};
pub use handover::{Handover, ProgressRow, Warning, handover};
pub use list::{Filter, Listing, LogRow, Row, list};
pub use next::{NextRow, RequirementLine, next};
pub use publish::publish;
pub use show::{EdgeLine, Shown, show};

/// How a need, question, or requirement reads its open/closed state.
fn open_or_closed(closed: bool) -> &'static str {
    if closed { "closed" } else { "open" }
}
