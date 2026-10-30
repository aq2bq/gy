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
//!   without `to` is that one day, and a `to` before its `from` is 400. A broken
//!   day is 400.
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
//!   direction it is stored (`relation` is its name, un-inverted), in the order
//!   of the nodes they leave.
//! - `scopes`: the scopes the cut touches, most nodes first, ties by name, with
//!   the count of each kind it holds and how many of those are open (`open`,
//!   `filed`, `approved`, `unsatisfied`, `live`; a decision shows its total only
//!   as the model gives it no open/closed word of its own).
//! - `events`: only when `from`/`to` was given: what happened in the span,
//!   within the cut, in this order, zeros left out: `decided`,
//!   `question-closed`, `question-raised`, `requirement-approved`,
//!   `requirement-done`, `criterion-satisfied`, `need-closed`. A write that only
//!   rewrote a node is not an event.
//! - `weeks`: only when `capped`: the cut's moves, one entry per scope and
//!   Monday-start week — every scope the cut touches times every week of the
//!   span, zeros included, by scope then week. The span is the requested days
//!   when `from`/`to` was given, else the whole ledger's (see `/api/overview`).
//!   A move is an event as `events` counts them, plus a requirement's
//!   cancellation and a node's birth (its `created`, or the instant it was
//!   first asked / filed / decided), so the grid says when the work happened,
//!   not only what closed.
use crate::http::{Request, Response};
use crate::snapshot::{Act, Snapshot, day_of, instant, week_start, weeks_between};
use chrono::NaiveDate;
use gy_ledger::{NodeKind, Repository, Store, matches};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// The most nodes a picture may hold; past it the grid answers instead.
const CAP: usize = 900;

/// What the crossing screen asked for.
struct Filter {
    words: Vec<String>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    scopes: Vec<String>,
}

#[derive(Serialize)]
struct CrossNode {
    id: String,
    kind: NodeKind,
    scope: String,
    state: String,
    created: String,
    title: String,
    degree: usize,
}

#[derive(Serialize)]
struct CrossEdge {
    from: String,
    to: String,
    relation: &'static str,
}

#[derive(Serialize)]
struct KindCount {
    kind: NodeKind,
    total: usize,
    open: usize,
}

#[derive(Serialize)]
struct ScopeLine {
    name: String,
    kinds: Vec<KindCount>,
}

#[derive(Serialize)]
struct EventCount {
    what: &'static str,
    count: usize,
}

#[derive(Serialize)]
struct WeekCount {
    scope: String,
    from: String,
    moves: usize,
}

#[derive(Serialize)]
struct Cross {
    seq: u64,
    at: String,
    capped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    nodes: Option<Vec<CrossNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edges: Option<Vec<CrossEdge>>,
    scopes: Vec<ScopeLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    events: Option<Vec<EventCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weeks: Option<Vec<WeekCount>>,
}

pub fn answer<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    let filter = match Filter::read(req) {
        Ok(filter) => filter,
        Err(response) => return response,
    };
    let snapshot = match Snapshot::read(repo) {
        Ok(snapshot) => snapshot,
        Err(error) => return Response::text(500, &error.to_string()),
    };
    Response::json(&view(&snapshot, &filter))
}

impl Filter {
    fn read(req: &Request) -> std::result::Result<Self, Response> {
        let words = req
            .param("q")
            .unwrap_or_default()
            .split_whitespace()
            .map(|word| word.to_lowercase())
            .collect();
        let from = day(req.param("from"), "from")?;
        let to = day(req.param("to"), "to")?;
        if let (Some(from), Some(to)) = (from, to) {
            if to < from {
                return Err(Response::text(400, "to is before from"));
            }
        }
        Ok(Self {
            words,
            from,
            to: to.or(from),
            scopes: asked_scopes(req),
        })
    }

    /// Whether the screen narrowed anything: nothing narrowed is the grid.
    fn any(&self) -> bool {
        !self.words.is_empty() || self.from.is_some() || !self.scopes.is_empty()
    }

    /// The days asked for, both ends included.
    fn span(&self) -> Option<(NaiveDate, NaiveDate)> {
        self.from.map(|from| (from, self.to.unwrap_or(from)))
    }

    fn holds_scope(&self, scope: &str) -> bool {
        self.scopes.is_empty() || self.scopes.iter().any(|held| held == scope)
    }
}

/// One `from`/`to` value: a day, or nothing.
fn day(text: Option<&str>, name: &str) -> std::result::Result<Option<NaiveDate>, Response> {
    match text.map(str::trim).filter(|text| !text.is_empty()) {
        Some(text) => NaiveDate::parse_from_str(text, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| Response::text(400, &format!("{name} is not a day (YYYY-MM-DD)"))),
        None => Ok(None),
    }
}

/// Every `scope` value; `all` and an empty one narrow nothing.
fn asked_scopes(req: &Request) -> Vec<String> {
    let mut scopes: Vec<String> = req
        .query
        .iter()
        .filter(|(name, _)| name == "scope")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "all")
        .collect();
    scopes.sort();
    scopes.dedup();
    scopes
}

fn view(snap: &Snapshot, filter: &Filter) -> Cross {
    let cut = cut(snap, filter);
    let capped = !filter.any() || cut.len() > CAP;
    let (nodes, edges) = if capped {
        (None, None)
    } else {
        let (nodes, edges) = picture(snap, &cut);
        (Some(nodes), Some(edges))
    };
    Cross {
        seq: snap.seq,
        at: crate::snapshot::rfc3339(snap.at),
        capped,
        nodes,
        edges,
        scopes: scope_lines(snap, &cut),
        events: filter.span().map(|span| events(snap, &cut, span)),
        weeks: capped.then(|| weeks(snap, &cut, filter.span())),
    }
}

/// The nodes the filters keep.
fn cut(snap: &Snapshot, filter: &Filter) -> Vec<usize> {
    (0..snap.nodes().len())
        .filter(|at| filter.holds_scope(snap.node(*at).scope()))
        .filter(|at| {
            filter
                .words
                .iter()
                .all(|word| matches(snap.node(*at), word).is_some())
        })
        .filter(|at| in_span(snap, *at, filter.span()))
        .collect()
}

/// Whether one of a node's moves falls in the span; no span keeps everything.
fn in_span(snap: &Snapshot, at: usize, span: Option<(NaiveDate, NaiveDate)>) -> bool {
    let Some((from, to)) = span else {
        return true;
    };
    snap.moves(at)
        .iter()
        .any(|moved| (from..=to).contains(&day_of(moved.at)))
}

/// The cut's nodes and the edges between them, each with its degree.
fn picture(snap: &Snapshot, cut: &[usize]) -> (Vec<CrossNode>, Vec<CrossEdge>) {
    let held: HashSet<usize> = cut.iter().copied().collect();
    let mut order: Vec<usize> = cut.to_vec();
    order.sort_by(|left, right| {
        snap.node(*left)
            .scope()
            .cmp(snap.node(*right).scope())
            .then_with(|| {
                (snap.node(*left).kind() as usize).cmp(&(snap.node(*right).kind() as usize))
            })
            .then_with(|| created(snap, *right).cmp(&created(snap, *left)))
            .then_with(|| {
                snap.node(*left)
                    .id()
                    .to_string()
                    .cmp(&snap.node(*right).id().to_string())
            })
    });
    let mut edges = Vec::new();
    let mut degree: HashMap<usize, usize> = HashMap::new();
    for &at in &order {
        for edge in snap.node(at).links() {
            let Some(to) = snap.index_of(&edge.to.to_string()) else {
                continue;
            };
            if !held.contains(&to) {
                continue;
            }
            *degree.entry(at).or_default() += 1;
            *degree.entry(to).or_default() += 1;
            edges.push(CrossEdge {
                from: snap.node(at).id().to_string(),
                to: snap.node(to).id().to_string(),
                relation: edge.label.name(),
            });
        }
    }
    let nodes = order
        .into_iter()
        .map(|at| {
            let node = snap.node(at);
            CrossNode {
                id: node.id().to_string(),
                kind: node.kind(),
                scope: node.scope().to_string(),
                state: snap.state(at).unwrap_or_default(),
                created: node.created().to_string(),
                title: node.title().to_string(),
                degree: degree.get(&at).copied().unwrap_or(0),
            }
        })
        .collect();
    (nodes, edges)
}

fn created(snap: &Snapshot, at: usize) -> u64 {
    instant(snap.node(at).created()).unwrap_or(0)
}

/// The scopes the cut touches, each with its kinds, the fullest first.
fn scope_lines(snap: &Snapshot, cut: &[usize]) -> Vec<ScopeLine> {
    let mut by_scope: HashMap<&str, Vec<usize>> = HashMap::new();
    for &at in cut {
        by_scope.entry(snap.node(at).scope()).or_default().push(at);
    }
    let mut lines: Vec<ScopeLine> = by_scope
        .into_iter()
        .map(|(name, held)| {
            let kinds = NodeKind::ALL
                .into_iter()
                .filter_map(|kind| {
                    let of_kind: Vec<usize> = held
                        .iter()
                        .copied()
                        .filter(|at| snap.node(*at).kind() == kind)
                        .collect();
                    (!of_kind.is_empty()).then(|| KindCount {
                        kind,
                        total: of_kind.len(),
                        open: of_kind.iter().filter(|at| snap.is_open(**at)).count(),
                    })
                })
                .collect();
            ScopeLine {
                name: name.to_string(),
                kinds,
            }
        })
        .collect();
    lines.sort_by(|left, right| {
        let total = |line: &ScopeLine| line.kinds.iter().map(|kind| kind.total).sum::<usize>();
        total(right)
            .cmp(&total(left))
            .then_with(|| left.name.cmp(&right.name))
    });
    lines
}

/// What the cut did inside the span, by the act's own name.
fn events(snap: &Snapshot, cut: &[usize], span: (NaiveDate, NaiveDate)) -> Vec<EventCount> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for &at in cut {
        for moved in snap.moves(at) {
            let Some(act) = moved.act else {
                continue;
            };
            if (span.0..=span.1).contains(&day_of(moved.at)) {
                *counts.entry(act.name()).or_default() += 1;
            }
        }
    }
    Act::ALL
        .into_iter()
        .filter_map(|act| {
            counts.get(act.name()).map(|count| EventCount {
                what: act.name(),
                count: *count,
            })
        })
        .collect()
}

/// The grid: each scope of the cut against each week, moves counted inside the
/// span (or the whole ledger's when no day was asked for).
fn weeks(snap: &Snapshot, cut: &[usize], span: Option<(NaiveDate, NaiveDate)>) -> Vec<WeekCount> {
    let weeks = match span {
        Some((from, to)) => weeks_between(week_start(from), week_start(to)),
        None => snap.weeks(),
    };
    let mut scopes: Vec<&str> = cut.iter().map(|at| snap.node(*at).scope()).collect();
    scopes.sort_unstable();
    scopes.dedup();
    let mut lines = Vec::new();
    for scope in scopes {
        for week in &weeks {
            let to = *week + chrono::Days::new(7);
            let moves = cut
                .iter()
                .filter(|at| snap.node(**at).scope() == scope)
                .flat_map(|at| snap.moves(*at))
                .filter(|moved| {
                    let day = day_of(moved.at);
                    day >= *week && day < to
                })
                .count();
            lines.push(WeekCount {
                scope: scope.to_string(),
                from: week.format("%Y-%m-%d").to_string(),
                moves,
            });
        }
    }
    lines
}
