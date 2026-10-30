//! `GET /api/cross?q=&from=&to=&scope=`: a cut of the record, for the crossing
//! screen (d-c302, d-6d90). Filters combine with AND; `scope` may repeat.
//!
//! # The answer
//!
//! ```json
//! {
//!   "seq": 4853,
//!   "at": "2026-10-30T04:46:44Z",
//!   "capped": false,
//!   "nodes": [
//!     {
//!       "id": "n-1a2b",
//!       "kind": "Need",
//!       "scope": "crm",
//!       "state": "open",
//!       "created": "2026-09-11T00:00:00Z",
//!       "title": "…",
//!       "degree": 7
//!     }
//!   ],
//!   "edges": [{"from": "n-1a2b", "to": "r-3c4d", "relation": "filed-as"}],
//!   "scopes": [
//!     {
//!       "name": "crm",
//!       "kinds": [{"kind": "Need", "total": 8, "open": 5}]
//!     }
//!   ],
//!   "events": [{"what": "decided", "count": 3}],
//!   "weeks": [{"scope": "crm", "from": "2026-09-07", "moves": 12}]
//! }
//! ```
//!
//! # The filters
//!
//! - `q`: the words, split on whitespace, each one a case-insensitive substring
//!   of the id, an alias, the title, or the body — the same match the Web
//!   search uses (d-6a45). Every word must hit.
//! - `from`, `to`: `YYYY-MM-DD`, inclusive, a UTC day added to `to`. A node is
//!   in the cut when one of its events (below) falls in the span; a `from`
//!   without `to` is that one day. A broken day is 400.
//! - `scope`: a scope name, repeated for several. A node is in the cut when it
//!   is in one of them. Empty and `all` mean every scope.
//!
//! # The keys
//!
//! - `capped`: `true` when no filter was given, or when the cut holds more than
//!   900 nodes: the picture is not drawn and `nodes`/`edges` are absent.
//! - `nodes`: the cut, ordered by scope, then kind (need, question, decision,
//!   requirement, criterion), then `created` newest first, ties by id. `state`
//!   is the list view's word (`open`, `closed`, `done`, `filed`, `approved`,
//!   `cancelled`, `satisfied`, `unsatisfied`, `withdrawn`), and for a decision
//!   `live` or `superseded` — a decision has no word in the model, so this one
//!   is read from the graph: another decision's `supersedes` points at it.
//!   `degree` is how many of the returned `edges` touch the node.
//! - `edges`: every stored edge whose two ends are both in the cut, in the
//!   direction it is stored (`relation` is its name, un-inverted).
//! - `scopes`: the scopes the cut touches, most nodes first, ties by name, with
//!   the count of each kind and how many of those are open (`open`, `filed`,
//!   `approved`, `unsatisfied`, `live`; a decision shows its total only as the
//!   model gives it no open/closed word of its own).
//! - `events`: only when `from`/`to` was given: what happened in the span,
//!   within the cut, in this order, zeros left out: `decided`,
//!   `question-closed`, `question-raised`, `requirement-approved`,
//!   `requirement-done`, `criterion-satisfied`, `need-closed`. A write that only
//!   rewrote a node is not an event.
//! - `weeks`: only when `capped`: the cut's moves, one entry per scope and
//!   Monday-start week — every scope the cut touches times every week of the
//!   span, zeros included. The span is the requested days when `from`/`to` was
//!   given, else the whole ledger's (see `/api/overview`). A move is an event
//!   as `events` counts them, plus a requirement's cancellation and a node's
//!   birth (its `created`, or the instant it was first asked / filed /
//!   decided), so the grid says when the work happened, not only what closed.
use crate::http::{Request, Response};
use gy_ledger::{Repository, Store};

pub fn answer<S: Store>(_repo: &Repository<S>, _req: &Request) -> Response {
    Response::text(501, "not implemented yet")
}
