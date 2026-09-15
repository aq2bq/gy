//! GET /api/labels?ids=a,b,c: the alias and title of the nodes a picture is
//! showing (n-4cd8). The nodes themselves stay in the ledger.
use crate::http::{Request, Response};
use gy_ledger::{Repository, Store};
use serde::Serialize;
use std::collections::BTreeMap;

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
    let mut labels = BTreeMap::new();
    for id in ids {
        let Ok(node) = repo.resolve(id) else { continue };
        let Ok(Some(node)) = repo.get(&node) else {
            continue;
        };
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
