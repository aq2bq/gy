//! GET /api/wait?after=<seq>: hold the request open until the ledger has a
//! later write (n-ff71). The waiting lives in `crate::watch`; this file only
//! shapes the answer.
use crate::http::{Request, Response};
use crate::watch::Watch;
use serde::Serialize;
use std::time::Duration;

/// How long one request may wait before it answers the sequence it has.
pub const TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Serialize)]
struct Seq {
    seq: u64,
}

pub fn wait(watch: &Watch, req: &Request) -> Response {
    let after = match req.param("after") {
        Some(text) => match text.parse::<u64>() {
            Ok(seq) => seq,
            Err(_) => return Response::text(400, "after is not a sequence"),
        },
        None => 0,
    };
    Response::json(&Seq {
        seq: watch.wait_past(after, TIMEOUT),
    })
}
