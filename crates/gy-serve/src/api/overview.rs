//! `GET /api/overview`: what the first screen asks, the shape of the progress
//! (d-e239, d-6d90). It reads a ledger whole, judges with gy-ledger's own
//! derivations (`model/derive.rs`), and places nothing.
//!
//! # The answer
//!
//! Instants are RFC 3339 UTC (`2026-10-30T04:46:44Z`), days are
//! `YYYY-MM-DD`, and `seq`/`at` name the point in the log the answer reads,
//! as in `/api/now`.
//!
//! ```json
//! {
//!   "seq": 4853,
//!   "at": "2026-10-30T04:46:44Z",
//!   "counts": {
//!     "open_needs": 48,
//!     "blocked_needs": 20,
//!     "open_questions": 45,
//!     "unmet_criteria": 62,
//!     "closed_needs": 218
//!   },
//!   "weeks": [
//!     {"from": "2026-09-07", "opened": 12, "done": 7}
//!   ],
//!   "scopes": [
//!     {
//!       "name": "crm",
//!       "open": 3,
//!       "closed": 40,
//!       "blocked": 1,
//!       "pulse": [{"day": "2026-10-01", "writes": 4}]
//!     }
//!   ],
//!   "needs": [
//!     {
//!       "id": "n-65c9",
//!       "scope": "crm",
//!       "title": "…",
//!       "stage": "before",
//!       "blocked": true,
//!       "last_moved": "2026-10-12T03:00:00Z",
//!       "requirements": {"filed": 1, "approved": 0, "done": 2, "cancelled": 0},
//!       "criteria": {"satisfied": 1, "unsatisfied": 1, "withdrawn": 0},
//!       "waiting": [
//!         {"id": "q-164b", "kind": "Question", "title": "…", "decider": "マスター"}
//!       ]
//!     }
//!   ],
//!   "blockers": [
//!     {
//!       "id": "q-164b",
//!       "kind": "Question",
//!       "scope": "crm",
//!       "title": "…",
//!       "decider": "マスター",
//!       "needs": ["n-b0f5", "n-da7f"],
//!       "via": ["n-65c9"]
//!     }
//!   ],
//!   "deciders": [{"name": "マスター", "count": 34}],
//!   "overdue": [
//!     {"id": "ac-1a2b", "scope": "crm", "title": "…", "due": "2026-10-01",
//!      "responsible": "…", "then": "…"}
//!   ],
//!   "unheld": [
//!     {"id": "q-164b", "kind": "Question", "scope": "crm", "state": "open", "title": "…"}
//!   ]
//! }
//! ```
//!
//! # The keys
//!
//! - `counts`: `open_needs` (a need the graph derives `open`), `blocked_needs`
//!   (an open need with at least one open wait), `open_questions` (a question
//!   still open), `unmet_criteria` (a criterion derived `unsatisfied`; a
//!   withdrawn one is not counted), `closed_needs` (every need that is not
//!   open: derived `done` or closed by hand).
//! - `weeks`: Monday-start weeks, from the week of the earliest instant the
//!   ledger holds (a write or a node's `created`) through the week of the
//!   latest. Every week of that span is present, zeros included. `opened`
//!   counts the needs created in the week; `done` the needs that reached
//!   `done` or were closed in it (see "When a need moved").
//! - `scopes`: every scope the ledger holds, the one with the most open needs
//!   first, ties by name. `open`/`closed`/`blocked` count its needs as
//!   `counts` does. `pulse` is its writes: one entry per day that has any, in
//!   the 49 days ending on the day of the answer's last write, counted once per
//!   write for each scope that write touched. Days without a write are left
//!   out; the screen draws the zeros.
//! - `needs`: every open need, needs with a wait first, then the one that moved
//!   last, newest first, ties by id. `stage` is `before` (着手前: no
//!   requirement yet), `build` (作っている: a requirement is filed or
//!   approved), `verify` (確かめ待ち: a requirement is done, none is still in
//!   progress) or `close` (閉じ待ち: its criteria are all satisfied, so no
//!   requirement is left to finish it). A need that is not open is not here.
//!   `blocked` is `true` when `waiting` is not empty. `last_moved` is the
//!   newest write to the need, its filed requirements, its targeted criteria,
//!   the things it waits on, or any node pointing at it. `requirements` counts
//!   the requirements it filed, by their state, and `criteria` counts the
//!   criteria it targets, each as the model derives it (`satisfied`,
//!   `unsatisfied`, `withdrawn`). `waiting` is what holds it: the open questions
//!   and requirements it `waits-on` and the open needs it `depends-on`, each
//!   `{id, kind, title, decider?}` — `decider` is present only for a question.
//! - `blockers`: the roots of the waits. A root is a waiting question, a
//!   waiting requirement, or a need that waits on nothing while another need
//!   depends on it; a need blocked only through the needs it depends on is not
//!   a root but a `via` step. One entry per root, the root blocking the most
//!   needs first, ties by id. `needs` is every open need this root holds;
//!   `via` is the chain of needs between those needs and the root, outermost
//!   first, empty when the root holds a need directly. `decider` is present
//!   only for a question.
//! - `deciders`: the open questions by the person who decides them, most first,
//!   ties by name. The name is the record's own, so the spellings it mixes
//!   (`マスター`, `master`) stay apart; a question that names none counts under
//!   `""`.
//! - `overdue`: the criteria whose `due` has come and that are still
//!   `unsatisfied` — the same wait `handover` dates (r-e9ca) — earliest date
//!   first, ties by id. `due`, `responsible`, and `then` are the node's own
//!   words. A withdrawn criterion is not overdue: nobody waits for it any more.
//! - `unheld`: what nobody is working on, questions first then criteria, newest
//!   first within each kind, ties by id. That is every open question no node
//!   `waits-on` and no node `raised` (gy-ledger's `unwaited`), and every
//!   `unsatisfied` criterion that no open need and no in-progress requirement
//!   targets. A withdrawn criterion is not here either.
//!
//! # When a need moved
//!
//! The model stamps a need's close with its `by` and evidence but no instant,
//! and a question's closure the same way, so those two take the instant of the
//! newest write to the node itself. A requirement's approval, completion and
//! cancellation carry their own instants, and so does a criterion's
//! `satisfied_at`. A need derived `done` moved when the last of those
//! happened, or, when the record carries none of them, at the newest write to
//! the need. Reads the model already records are the only source: an edit that
//! changes nothing a reader sees still counts as a write to `last_moved`, but
//! it never becomes an event of its own.
use crate::http::Response;
use gy_ledger::{Repository, Store};

pub fn answer<S: Store>(_repo: &Repository<S>) -> Response {
    Response::text(501, "not implemented yet")
}
