//! `GET /api/node/<id>/path?depth=2|3`: the thread around one node, for the
//! detail screen (d-e239, d-6d90). Left is where the node came from, right is
//! what it works on; one direction per relation, so the thread always reads
//! from cause to effect and never needs a force layout.
//!
//! # The answer
//!
//! ```json
//! {
//!   "seq": 4853,
//!   "at": "2026-10-30T04:46:44Z",
//!   "depth": 2,
//!   "center": {
//!     "id": "n-65c9", "kind": "Need", "scope": "crm",
//!     "state": "open", "title": "…"
//!   },
//!   "up": [
//!     {
//!       "nodes": [
//!         {
//!           "id": "q-164b", "kind": "Question", "scope": "crm",
//!           "state": "open", "title": "…",
//!           "relation": "waits-on", "parent": "n-65c9", "mark": "…"
//!         }
//!       ],
//!       "hidden": 0
//!     }
//!   ],
//!   "down": []
//! }
//! ```
//!
//! # The keys
//!
//! - `depth`: 2 (the default) or 3; any other value is 400. An unknown id is
//!   404.
//! - `center`: the node asked for, as the list view reads it.
//! - `up`: one entry per step to the left, `up[0]` one step away. A step's
//!   `nodes` are at most ten, the rest counted in `hidden`; a step with nothing
//!   in it ends the walk.
//! - `down`: the same to the right.
//! - A step's nodes are ordered: open first (`open`, `filed`, `approved`,
//!   `unsatisfied`, `live`), then `relation`, then kind (need, question,
//!   decision, requirement, criterion), then `created` newest first, ties by id.
//! - `state` is the list view's word, and for a decision `live` or
//!   `superseded` (see `/api/cross`). `relation` is the stored name of the edge
//!   that joins the node to its `parent`, un-inverted; `mark` is the passage the
//!   edge names, absent when it names none.
//! - `parent` is the id of the node this one hangs from: the center at the first
//!   step, a node of the previous step after that. A node appears once per
//!   direction, so a diamond keeps its first way in.
//!
//! # Which way a relation points
//!
//! Upstream is the `to` end of `depends-on`, `relies-on`, `waits-on`, `serves`
//! and `spawned-by`, and the `from` end of every other relation. That puts a
//! need's prerequisites and the decision that serves it on its left, a
//! requirement's need on its left, a criterion's needs and requirements on its
//! left, an old decision on the left of the one that narrows it, and a
//! question on the left of the decision that closes it; and it puts the
//! requirements a need filed, the criteria it targets, and the decisions and
//! questions it led to on the right.
use crate::http::Response;
use gy_ledger::{Repository, Store};

pub fn answer<S: Store>(_repo: &Repository<S>, _id: &str, _req: &crate::http::Request) -> Response {
    Response::text(501, "not implemented yet")
}
