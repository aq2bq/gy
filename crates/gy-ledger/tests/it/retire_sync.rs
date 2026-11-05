//! Retirement and concurrent links are rejudged against a real pair of copies.
use gy_ledger::{
    DecisionScope, FileStore, FormatVersion, Link, Node, NodeId, NodeKind, Operation, Relation,
    Repository, Retire, Store, format, log, sync,
};
use std::path::Path;
use std::process::Command;
fn git(dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap()
}
fn repo(dir: &Path) -> Repository<FileStore> {
    Repository::new(FileStore::open_with(dir, |_| Some("gy-dev-dev".into())).unwrap())
}
fn decision(hash: &str) -> Node {
    Node::decision(
        NodeId::from_hash(NodeKind::Decision, hash).unwrap(),
        "a",
        "2026-11-05T00:00:00Z",
        "earlier record",
        DecisionScope::recorded("scope").unwrap(),
    )
    .unwrap()
}
fn retire(r: &mut Repository<FileStore>) {
    Retire {
        ids: vec!["d-0001".into()],
        reason: "process record".into(),
        revoke: false,
    }
    .run(r)
    .unwrap();
}
fn link(r: &mut Repository<FileStore>) {
    let mut live = r.get(&decision("0002").id().clone()).unwrap().unwrap();
    live.link(
        Link::new(
            live.id().clone(),
            Relation::Completes,
            decision("0001").id().clone(),
        )
        .unwrap(),
    );
    r.transaction("link", "test", |r| r.put(&live)).unwrap();
}
#[test]
fn concurrent_link_is_rejected_after_remote_retirement_and_format_precedes_write() {
    crate::common::ident();
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    std::fs::create_dir_all(&remote).unwrap();
    git(&remote, &["init", "--bare", "-q"]);
    let url = format!("file://{}", remote.display());
    let one = temp.path().join("one");
    let two = temp.path().join("two");
    format::write(&one, FormatVersion::CURRENT).unwrap();
    let mut r = repo(&one);
    r.transaction("seed", "test", |r| {
        r.put(&decision("0001"))?;
        r.put(&decision("0002"))
    })
    .unwrap();
    drop(r);
    sync(&one, &url).unwrap();
    sync(&two, &url).unwrap();
    link(&mut repo(&two));
    retire(&mut repo(&one));
    sync(&one, &url).unwrap();
    assert_eq!(git(&remote, &["show", "main:format"]), "5\n");
    assert!(git(&remote, &["log", "--format=%B", "main~1", "-1"]).contains("Gy-Format: 5"));
    let report = sync(&two, &url).unwrap();
    assert_eq!(report.rejected, Some(1));
    let r = repo(&two);
    assert_eq!(r.all().unwrap().len(), 1);
    assert!(r.store().rejected().join("\n").contains("retired"));
    assert!(
        std::fs::read_to_string(two.join("rejected.jsonl"))
            .unwrap()
            .contains("d-0001")
    );
    let events = log::read(&two).unwrap().0;
    assert_eq!(events.last().unwrap().seq, 2);
}
#[test]
fn concurrent_retirement_is_rejected_if_remote_adds_a_boundary() {
    crate::common::ident();
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    std::fs::create_dir_all(&remote).unwrap();
    git(&remote, &["init", "--bare", "-q"]);
    let url = format!("file://{}", remote.display());
    let one = temp.path().join("one");
    let two = temp.path().join("two");
    format::write(&one, FormatVersion::CURRENT).unwrap();
    let mut r = repo(&one);
    r.transaction("seed", "test", |r| {
        r.put(&decision("0001"))?;
        r.put(&decision("0002"))
    })
    .unwrap();
    drop(r);
    sync(&one, &url).unwrap();
    sync(&two, &url).unwrap();
    retire(&mut repo(&two));
    link(&mut repo(&one));
    sync(&one, &url).unwrap();
    let report = sync(&two, &url).unwrap();
    assert_eq!(report.rejected, Some(1));
    let r = repo(&two);
    assert_eq!(r.all().unwrap().len(), 2);
    let notice = r.store().rejected().join("\n");
    for part in ["remaining edge", "d-0001", "completes", "d-0002"] {
        assert!(notice.contains(part), "{notice}");
    }
}
