use gy_ledger::link::Link;
use gy_ledger::{
    CriterionAdd, Edit, FileStore, FormatVersion, NeedAdd, Operation, Relation, Repository, Undo,
    file, format,
};
use gy_serve::http::Request;
use gy_serve::server::{Opened, Opener, answer};
use std::path::{Path, PathBuf};

/// A throwaway directory under the system's temp root (no extra dependency).
fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gy-serve-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Six writes on a real ledger directory: a criterion, two needs, a link, an
/// edit, and an undo of that edit.
fn ledger(dir: &Path) {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let store = FileStore::open_with(dir, |_| Some("piko".to_string())).unwrap();
    let mut repo = Repository::new(store).with_scopes(vec!["a".to_string()]);
    let criterion = CriterionAdd {
        scope: "a".to_string(),
        title: "measurable".to_string(),
    }
    .run(&mut repo)
    .unwrap()
    .id
    .unwrap();
    let first = NeedAdd {
        scope: "a".to_string(),
        title: "first".to_string(),
        targets: vec![criterion.clone()],
        spawned_by: None,
    }
    .run(&mut repo)
    .unwrap()
    .id
    .unwrap();
    let second = NeedAdd {
        scope: "a".to_string(),
        title: "second".to_string(),
        targets: vec![criterion],
        spawned_by: None,
    }
    .run(&mut repo)
    .unwrap()
    .id
    .unwrap();
    Link {
        from: second,
        relation: Relation::DependsOn,
        to: first.clone(),
        mark: None,
        remove: false,
    }
    .run(&mut repo)
    .unwrap();
    Edit {
        id: first,
        reason: "tidy".to_string(),
        title: Some("first, tidied".to_string()),
        body: None,
        set: Vec::new(),
        append: Vec::new(),
    }
    .run(&mut repo)
    .unwrap();
    Undo {
        reason: "wrong".to_string(),
    }
    .run(&mut repo)
    .unwrap();
}

/// The opener gy serve builds: the head, or the asked point in the log.
fn opener(dir: PathBuf) -> Opener {
    Box::new(move |at| {
        let lookup = |_: &str| Some("piko".to_string());
        match at {
            None => {
                FileStore::open_with(&dir, lookup).map(|store| Opened::Now(Repository::new(store)))
            }
            Some(seq) => {
                file::open_at(&dir, seq, lookup).map(|store| Opened::At(Repository::new(store)))
            }
        }
    })
}

fn get(path: &str, query: Option<&str>) -> Request {
    Request::new("GET", path, query, &[])
}

fn json(open: &Opener, path: &str, query: Option<&str>) -> serde_json::Value {
    serde_json::from_slice(&answer(open, &get(path, query)).body).unwrap()
}

fn rows(value: &serde_json::Value) -> usize {
    value["rows"].as_array().unwrap().len()
}

fn total(value: &serde_json::Value, kind: &str) -> u64 {
    value["kinds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["kind"] == kind)
        .unwrap()["total"]
        .as_u64()
        .unwrap()
}

#[test]
fn at_reads_an_earlier_point_in_the_log() {
    let dir = tempdir("at");
    ledger(&dir);
    let open = opener(dir.clone());

    let full = json(&open, "/api/list", Some("kind=Need"));
    let early = json(&open, "/api/list", Some("kind=Need&at=2"));
    assert!(rows(&early) < rows(&full), "{early} {full}");
    assert_eq!(json(&open, "/api/list", Some("kind=Need&at=999")), full);

    let shell = json(&open, "/api/shell", None);
    let early_shell = json(&open, "/api/shell", Some("at=2"));
    assert_eq!(shell["seq"], 6);
    assert_eq!(early_shell["seq"], 2);
    assert!(total(&early_shell, "Need") < total(&shell, "Need"));

    assert_eq!(answer(&open, &get("/api/list", Some("at=abc"))).status, 400);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ticks_lists_one_tick_per_write() {
    let dir = tempdir("ticks");
    ledger(&dir);
    let open = opener(dir.clone());

    let ticks = json(&open, "/api/ticks", None);
    let list = ticks["ticks"].as_array().unwrap();
    assert_eq!(list.len(), 6);
    assert_eq!(ticks["max"], list.last().unwrap()["seq"]);
    assert_eq!(list[0]["kind"], "criterion");
    assert_eq!(list[1]["kind"], "need");
    assert_eq!(list[3]["kind"], "link");
    assert_eq!(list[4]["kind"], "edit");
    assert_eq!(list[5]["kind"], "undo");

    // The scrubber always sees the whole log, whatever `at` says.
    assert_eq!(json(&open, "/api/ticks", Some("at=1")), ticks);
    let _ = std::fs::remove_dir_all(&dir);
}
