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
use crate::http::{Request, Response};
use crate::snapshot::{Snapshot, instant};
use gy_ledger::{NodeKind, Relation, Repository, Store};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashSet;

/// The most boxes one step shows; the rest are counted.
const CAP: usize = 10;

#[derive(Serialize)]
struct Center {
    id: String,
    kind: NodeKind,
    scope: String,
    state: String,
    title: String,
}

#[derive(Serialize)]
struct StepNode {
    id: String,
    kind: NodeKind,
    scope: String,
    state: String,
    title: String,
    relation: &'static str,
    parent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mark: Option<String>,
}

#[derive(Serialize)]
struct Step {
    nodes: Vec<StepNode>,
    hidden: usize,
}

#[derive(Serialize)]
struct Path {
    seq: u64,
    at: String,
    depth: usize,
    center: Center,
    up: Vec<Step>,
    down: Vec<Step>,
}

pub fn answer<S: Store>(repo: &Repository<S>, id: &str, req: &Request) -> Response {
    let depth = match depth(req) {
        Ok(depth) => depth,
        Err(response) => return response,
    };
    let resolved = match repo.resolve(id) {
        Ok(resolved) => resolved.to_string(),
        Err(_) => return Response::text(404, "not found"),
    };
    let snapshot = match Snapshot::read(repo) {
        Ok(snapshot) => snapshot,
        Err(error) => return Response::text(500, &error.to_string()),
    };
    let Some(center) = snapshot.index_of(&resolved) else {
        return Response::text(404, "not found");
    };
    Response::json(&view(&snapshot, center, depth))
}

/// `depth=2|3`, two by default; anything else is 400.
fn depth(req: &Request) -> std::result::Result<usize, Response> {
    match req.param("depth") {
        None | Some("") | Some("2") => Ok(2),
        Some("3") => Ok(3),
        Some(text) => Err(Response::text(400, &format!("depth is 2 or 3, not {text}"))),
    }
}

fn view(snap: &Snapshot, center: usize, depth: usize) -> Path {
    let node = snap.node(center);
    Path {
        seq: snap.seq,
        at: crate::snapshot::rfc3339(snap.at),
        depth,
        center: Center {
            id: node.id().to_string(),
            kind: node.kind(),
            scope: node.scope().to_string(),
            state: snap.state(center).unwrap_or_default(),
            title: node.title().to_string(),
        },
        up: walk(snap, center, true, depth),
        down: walk(snap, center, false, depth),
    }
}

/// One direction of the thread: each step's nodes, and how many the cap left
/// out. A node already seen on this side keeps its first way in.
fn walk(snap: &Snapshot, center: usize, up: bool, depth: usize) -> Vec<Step> {
    let mut seen = HashSet::from([center]);
    let mut front = vec![center];
    let mut steps = Vec::new();
    for _ in 0..depth {
        let mut found: Vec<Found> = Vec::new();
        for &parent in &front {
            for arrival in neighbours(snap, parent, up) {
                if seen.insert(arrival.at) {
                    found.push(Found {
                        at: arrival.at,
                        parent,
                        relation: arrival.relation,
                        mark: arrival.mark,
                    });
                }
            }
        }
        if found.is_empty() {
            break;
        }
        found.sort_by(|left, right| order(snap, left, right));
        let hidden = found.len().saturating_sub(CAP);
        found.truncate(CAP);
        front = found.iter().map(|found| found.at).collect();
        steps.push(Step {
            nodes: found.into_iter().map(|found| line(snap, found)).collect(),
            hidden,
        });
    }
    steps
}

/// One neighbour of a node, and how it hangs.
struct Also {
    at: usize,
    relation: Relation,
    mark: Option<String>,
}

/// A node found one step away, with the box it hangs from.
struct Found {
    at: usize,
    parent: usize,
    relation: Relation,
    mark: Option<String>,
}

fn line(snap: &Snapshot, found: Found) -> StepNode {
    let node = snap.node(found.at);
    StepNode {
        id: node.id().to_string(),
        kind: node.kind(),
        scope: node.scope().to_string(),
        state: snap.state(found.at).unwrap_or_default(),
        title: node.title().to_string(),
        relation: found.relation.name(),
        parent: snap.node(found.parent).id().to_string(),
        mark: found.mark,
    }
}

/// Whether a relation's `to` end is upstream of its `from` end.
fn upstream(relation: Relation) -> bool {
    matches!(
        relation,
        Relation::DependsOn
            | Relation::ReliesOn
            | Relation::WaitsOn
            | Relation::Serves
            | Relation::SpawnedBy
            | Relation::Narrows
            | Relation::Widens
            | Relation::Supersedes
            | Relation::Completes
    )
}

/// The node's neighbours on one side: the `to` ends of its stored edges and the
/// `from` ends of the edges arriving at it.
fn neighbours(snap: &Snapshot, at: usize, up: bool) -> Vec<Also> {
    let mut found = Vec::new();
    for edge in snap.node(at).links() {
        if upstream(edge.label) != up {
            continue;
        }
        let Some(other) = snap.index_of(&edge.to.to_string()) else {
            continue;
        };
        found.push(Also {
            at: other,
            relation: edge.label,
            mark: edge.mark.clone(),
        });
    }
    for arrival in snap.arrivals(at) {
        if upstream(arrival.relation) == up {
            continue;
        }
        found.push(Also {
            at: arrival.from,
            relation: arrival.relation,
            mark: arrival.mark.clone(),
        });
    }
    found
}

/// Open first, then the relation, then the kind, then the newest.
fn order(snap: &Snapshot, left: &Found, right: &Found) -> Ordering {
    snap.is_open(right.at)
        .cmp(&snap.is_open(left.at))
        .then_with(|| left.relation.name().cmp(right.relation.name()))
        .then_with(|| {
            (snap.node(left.at).kind() as usize).cmp(&(snap.node(right.at).kind() as usize))
        })
        .then_with(|| created(snap, right.at).cmp(&created(snap, left.at)))
        .then_with(|| {
            snap.node(left.at)
                .id()
                .to_string()
                .cmp(&snap.node(right.at).id().to_string())
        })
}

fn created(snap: &Snapshot, at: usize) -> u64 {
    instant(snap.node(at).created()).unwrap_or(0)
}
