use crate::common::ident;
use gy_ledger::{
    DecisionScope, FileStore, FormatVersion, Link, Node, NodeId, NodeKind, Operation, Relation,
    Repository, format, log, show, sync,
};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo(path: &Path) -> Repository<FileStore> {
    let actor = if path.file_name().unwrap() == "b" {
        "peer"
    } else {
        "piko"
    };
    Repository::new(FileStore::open_with(path, |_| Some(actor.into())).unwrap())
}
fn id(hash: &str) -> NodeId {
    NodeId::from_hash(NodeKind::Decision, hash).unwrap()
}
fn decision(hash: &str) -> Node {
    let mut n = Node::decision(
        id(hash),
        "a",
        "2026-09-15T00:00:00Z",
        "unchanged title",
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap();
    n.set_body("alpha beta gamma");
    n
}
fn pair(dir: &Path) -> (PathBuf, PathBuf, String) {
    ident();
    let a = dir.join("a");
    let b = dir.join("b");
    let bare = dir.join("remote.git");
    assert!(
        Command::new("git")
            .args(["init", "-q", "--bare"])
            .arg(&bare)
            .status()
            .unwrap()
            .success()
    );
    format::write(&a, FormatVersion::CURRENT).unwrap();
    let mut r = repo(&a);
    let need = Node::need(
        NodeId::from_hash(NodeKind::Need, "0001").unwrap(),
        "a",
        "2026-09-15T00:00:00Z",
        "purpose",
    )
    .unwrap();
    r.transaction("seed", "test", |r| {
        r.put(&need)?;
        for hash in ["0001", "0002", "0003"] {
            let mut d = decision(hash);
            d.link(Link::new(d.id().clone(), Relation::Serves, need.id().clone()).unwrap());
            r.put(&d)?;
        }
        Ok(())
    })
    .unwrap();
    drop(r);
    let url = format!("file://{}", bare.display());
    sync(&a, &url).unwrap();
    sync(&b, &url).unwrap();
    (a, b, url)
}
fn link(path: &Path, to: &str, relation: Relation, mark: &str, remove: bool) {
    gy_ledger::link::Link {
        from: id("0001"),
        relation,
        to: id(to),
        mark: Some(mark.into()),
        remove,
    }
    .run(&mut repo(path))
    .unwrap();
}
fn edit(path: &Path, text: &str) {
    gy_ledger::Edit {
        id: id("0001"),
        reason: "edit".into(),
        title: None,
        body: Some(text.into()),
        set: vec![],
        append: vec![],
    }
    .run(&mut repo(path))
    .unwrap();
}
fn marks(path: &Path) -> Vec<String> {
    repo(path)
        .get(&id("0001"))
        .unwrap()
        .unwrap()
        .links()
        .iter()
        .filter(|e| e.label == Relation::Narrows)
        .map(|e| e.mark.clone().unwrap())
        .collect()
}
fn json(path: &Path) -> serde_json::Value {
    serde_json::to_value(show(&repo(path), &["d-0001".into()], true).unwrap()).unwrap()
}

#[test]
fn different_passages_preserve_remote_values_and_all_local_additions() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, url) = pair(dir.path());
    link(&a, "0002", Relation::Narrows, "alpha", false);
    link(&a, "0002", Relation::Narrows, "gamma", false);
    sync(&a, &url).unwrap();
    let remote_node = repo(&a).get(&id("0001")).unwrap().unwrap();
    link(&b, "0002", Relation::Narrows, "beta", false);
    let report = sync(&b, &url).unwrap();
    assert!(report.rejected.is_none(), "{report:?}");
    assert_eq!(marks(&b), ["alpha", "gamma", "beta"]);
    let mut merged = repo(&b).get(&id("0001")).unwrap().unwrap();
    merged.unlink(merged.links().len() - 1);
    assert_eq!(merged, remote_node);
    sync(&a, &url).unwrap();
    assert_eq!(json(&a), json(&b));
    // The already-merged event must remain identifiable on retry.
    sync(&b, &url).unwrap();
    assert_eq!(marks(&b).len(), 3);
}

#[test]
fn successive_local_additions_use_the_original_local_history() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, url) = pair(dir.path());
    link(&a, "0002", Relation::Narrows, "alpha", false);
    sync(&a, &url).unwrap();
    link(&b, "0002", Relation::Narrows, "beta", false);
    link(&b, "0002", Relation::Narrows, "gamma", false);
    let report = sync(&b, &url).unwrap();
    assert!(report.rejected.is_none(), "{report:?}");
    assert_eq!(marks(&b), ["alpha", "beta", "gamma"]);
    sync(&a, &url).unwrap();
    assert_eq!(json(&a), json(&b));
}

#[test]
fn same_passage_is_rejected_with_the_original_event_and_remote_author() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b, url) = pair(dir.path());
    link(&a, "0002", Relation::Narrows, "alpha", false);
    link(&b, "0002", Relation::Narrows, "alpha", false);
    let original = log::read(&b).unwrap().0.last().unwrap().clone();
    sync(&a, &url).unwrap();
    assert_eq!(sync(&b, &url).unwrap().rejected, Some(1));
    assert_eq!(marks(&b), ["alpha"]);
    let rejected: serde_json::Value = serde_json::from_str(
        std::fs::read_to_string(b.join("rejected.jsonl"))
            .unwrap()
            .lines()
            .last()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(rejected["event"], serde_json::to_value(original).unwrap());
    assert!(
        rejected["reason"]
            .as_str()
            .unwrap()
            .contains("already registered")
    );
    assert_eq!(rejected["by"]["actor"], "piko");
}

#[test]
fn other_touches_still_reject_even_if_an_edit_is_later_restored() {
    for case in [
        "body",
        "restored",
        "pair",
        "relation",
        "remove",
        "local_body",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let (a, b, url) = pair(dir.path());
        match case {
            "body" => edit(&a, "remote body"),
            "restored" => {
                edit(&a, "remote body");
                edit(&a, "alpha beta gamma");
            }
            "pair" => link(&a, "0003", Relation::Narrows, "alpha", false),
            "relation" => link(&a, "0002", Relation::Widens, "alpha", false),
            "remove" => {
                link(&a, "0002", Relation::Narrows, "alpha", false);
                link(&a, "0002", Relation::Narrows, "alpha", true);
            }
            "local_body" => {
                link(&a, "0002", Relation::Narrows, "alpha", false);
                edit(&b, "local body");
            }
            _ => unreachable!(),
        }
        sync(&a, &url).unwrap();
        link(&b, "0002", Relation::Narrows, "beta", false);
        let report = sync(&b, &url).unwrap();
        assert_eq!(
            report.rejected,
            Some(if case == "local_body" { 2 } else { 1 }),
            "{case}: {report:?}"
        );
        let rejected = std::fs::read_to_string(b.join("rejected.jsonl")).unwrap();
        assert!(
            rejected.contains("the remote changed d-0001"),
            "{case}: {rejected}"
        );
        assert!(!marks(&b).contains(&"beta".to_string()));
    }
}
