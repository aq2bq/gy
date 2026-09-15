//! `GET /api/shell?scope=<name>`: the numbers the sidebar draws (n-a493). The
//! counts come from the `list` view, so no status is re-decided here.
use crate::http::{Request, Response};
use gy_ledger::{Filter, Listing, NodeKind, Repository, Result, Row, Store};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Shell {
    seq: u64,
    at: String,
    scope: String,
    scopes: Vec<ScopeCount>,
    kinds: Vec<KindCount>,
}

#[derive(Debug, Serialize)]
struct ScopeCount {
    name: String,
    count: usize,
}

#[derive(Debug, Serialize)]
struct KindCount {
    kind: NodeKind,
    open: usize,
    total: usize,
}

pub fn shell<S: Store>(repo: &Repository<S>, req: &Request) -> Response {
    let all = match rows(repo) {
        Ok(rows) => rows,
        Err(error) => return Response::text(500, &error.to_string()),
    };
    let scope = req.param("scope").unwrap_or("all").to_string();
    let visible: Vec<&Row> = all
        .iter()
        .filter(|row| scope == "all" || row.scope == scope)
        .collect();
    let (seq, at) = last_write(repo);
    let kinds = NodeKind::ALL
        .into_iter()
        .map(|kind| kind_count(&visible, kind))
        .collect();
    Response::json(&Shell {
        seq,
        at,
        scope,
        scopes: scope_counts(&all),
        kinds,
    })
}

fn rows<S: Store>(repo: &Repository<S>) -> Result<Vec<Row>> {
    match gy_ledger::list(repo, &Filter::default())? {
        Listing::Nodes(rows) => Ok(rows),
        Listing::History(_) => Ok(Vec::new()),
    }
}

/// The shell's only status reading: a row is open when the model named it so
/// (need and question `open`, criterion `unsatisfied`, requirement `filed` or
/// `approved`; a need that is done, and a decision, are not open — d-8a24).
fn is_open(status: Option<&str>) -> bool {
    matches!(status, Some("open" | "unsatisfied" | "filed" | "approved"))
}

fn kind_count(rows: &[&Row], kind: NodeKind) -> KindCount {
    let of_kind = rows.iter().copied().filter(|row| row.kind == kind);
    KindCount {
        kind,
        open: of_kind
            .clone()
            .filter(|row| is_open(row.status.as_deref()))
            .count(),
        total: of_kind.count(),
    }
}

/// Every scope the ledger holds, in name order, with its own count.
fn scope_counts(rows: &[Row]) -> Vec<ScopeCount> {
    let mut names: Vec<&str> = rows.iter().map(|row| row.scope.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    names
        .into_iter()
        .map(|name| ScopeCount {
            name: name.to_string(),
            count: rows.iter().filter(|row| row.scope == name).count(),
        })
        .collect()
}

/// The last write in the log, as a sequence and an RFC3339 time.
fn last_write<S: Store>(repo: &Repository<S>) -> (u64, String) {
    let last = repo.store().history().last();
    let at = chrono::DateTime::from_timestamp(last.map_or(0, |entry| entry.at) as i64, 0);
    (
        last.map_or(0, |entry| entry.seq),
        at.map(|time| time.to_rfc3339()).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::is_open;

    /// A need that is done is closed, not open (d-8a24).
    #[test]
    fn done_is_not_open() {
        for open in ["open", "unsatisfied", "filed", "approved"] {
            assert!(is_open(Some(open)), "{open}");
        }
        for closed in ["done", "closed", "cancelled", "satisfied"] {
            assert!(!is_open(Some(closed)), "{closed}");
        }
    }
}
