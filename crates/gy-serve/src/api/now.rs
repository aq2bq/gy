//! GET /api/now: the `now` view as JSON, with the time written out (n-688a).
//! The judgement lives in gy-ledger; this file only shapes the answer.
use crate::http::{Request, Response};
use gy_ledger::{LogRow, NodeRow, Repository, Store, Waiting, now};
use serde::Serialize;

/// The view with `at` as an RFC3339 time instead of epoch seconds.
#[derive(Serialize)]
struct Answer<'a> {
    seq: u64,
    at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<&'a str>,
    waiting: &'a [Waiting],
    in_progress: &'a [NodeRow],
    open_questions: &'a [NodeRow],
    unmet: &'a [NodeRow],
    recent: &'a [LogRow],
}

pub fn answer<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    let view = match now(repo, req.param("scope")) {
        Ok(view) => view,
        Err(error) => return Response::text(500, &error.to_string()),
    };
    let at = chrono::DateTime::from_timestamp(view.at as i64, 0)
        .map(|time| time.to_rfc3339())
        .unwrap_or_default();
    Response::json(&Answer {
        seq: view.seq,
        at,
        scope: view.scope.as_deref(),
        waiting: &view.waiting,
        in_progress: &view.in_progress,
        open_questions: &view.open_questions,
        unmet: &view.unmet,
        recent: &view.recent,
    })
}
