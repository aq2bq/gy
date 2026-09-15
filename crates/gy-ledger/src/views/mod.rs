//! Derived views: projections that read through the model and ops only, never
//! the store directly (D-76). N-57 brings show, N-58 list; next, handover, and
//! publish follow.
mod list;
mod show;

pub use list::{Filter, Listing, LogRow, Row, list};
pub use show::{EdgeLine, Shown, show};

/// How a need, question, or requirement reads its open/closed state.
fn open_or_closed(closed: bool) -> &'static str {
    if closed { "closed" } else { "open" }
}
