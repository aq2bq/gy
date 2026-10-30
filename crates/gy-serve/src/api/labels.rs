//! GET /api/labels?ids=a,b,c: the alias and title of the nodes a picture is
//! showing (n-4cd8). The nodes themselves stay in the ledger.
//!
//! The peers come from one read of the ledger, not one resolve per id: a
//! history page asks for two hundred ids at once, and a resolve apiece opened
//! the log two hundred times (r-0f99).
use crate::http::{Request, Response};
use gy_ledger::{Node, Repository, Store};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

const LIMIT: usize = 200;

#[derive(Serialize)]
struct Label {
    alias: Option<String>,
    title: String,
}

#[derive(Serialize)]
struct Labels {
    labels: BTreeMap<String, Label>,
}

pub fn labels<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    let ids: Vec<&str> = req
        .param("ids")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect();
    if ids.len() > LIMIT {
        return Response::text(400, "too many ids");
    }
    let all = repo.all().unwrap_or_default();
    let known = index(&all);
    let mut labels = BTreeMap::new();
    for id in ids {
        let Some(node) = known.get(id) else { continue };
        labels.insert(
            node.id().to_string(),
            Label {
                alias: node.aliases().first().map(|alias| alias.0.clone()),
                title: node.title().to_string(),
            },
        );
    }
    Response::json(&Labels { labels })
}

/// Every node by its id and by each of its aliases, from the one read.
fn index(all: &[Node]) -> HashMap<String, &Node> {
    let mut known = HashMap::with_capacity(all.len() * 2);
    for node in all {
        known.insert(node.id().to_string(), node);
        for alias in node.aliases() {
            known.insert(alias.0.clone(), node);
        }
    }
    known
}
