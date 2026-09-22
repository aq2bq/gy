//! n-557f: the gate at the commit entries (a normal write, undo) and the
//! readings that must never reach it.
use gy_ledger::{
    CriterionSatisfy, Error, FileStore, Filter, FormatVersion, Gate, Node, NodeId, NodeKind,
    Operation, Repository, Result, Undo, file, format, list, log, publish, show,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

/// A gate that refuses every write, for the commit entries.
#[derive(Debug, Default)]
struct Reject;
impl Gate for Reject {
    fn admit(&self, _before: &BTreeMap<String, Value>, _changes: &[log::Change]) -> Result<()> {
        Err(Error::invalid("the test gate refuses"))
    }
}

/// A gate that admits and counts every call, for the reading test.
#[derive(Debug, Default)]
struct Count(Arc<AtomicUsize>);
impl Gate for Count {
    fn admit(&self, _before: &BTreeMap<String, Value>, _changes: &[log::Change]) -> Result<()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn criterion(hash: &str) -> Node {
    Node::criterion(
        NodeId::from_hash(NodeKind::Criterion, hash).unwrap(),
        SCOPE,
        DATE,
        "a criterion",
    )
    .unwrap()
}

fn satisfy(id: &NodeId) -> CriterionSatisfy {
    CriterionSatisfy {
        id: id.clone(),
        evidence: "verified".into(),
        revoke: false,
    }
}

fn open(dir: &Path) -> Repository<FileStore> {
    Repository::new(FileStore::open_with(dir, |_| Some("piko".into())).unwrap())
}

fn gated(dir: &Path, gate: Arc<dyn Gate>) -> Repository<FileStore> {
    Repository::with_gate(
        FileStore::open_with(dir, |_| Some("piko".into())).unwrap(),
        gate,
    )
}

/// Seed one criterion through a normal repository and return its id.
fn seed(dir: &Path) -> NodeId {
    format::write(dir, FormatVersion::CURRENT).unwrap();
    let criterion = criterion("0001");
    let mut repo = open(dir);
    repo.transaction("seed", "test", |repo| repo.put(&criterion))
        .unwrap();
    criterion.id().clone()
}

fn lines(dir: &Path) -> (usize, u64) {
    let events = log::read(dir).unwrap().0;
    (events.len(), events.last().map_or(0, |event| event.seq))
}

#[test]
fn a_refused_write_leaves_the_log_and_the_sequence() {
    let temp = tempfile::tempdir().unwrap();
    let id = seed(temp.path());
    let before = lines(temp.path());
    let mut repo = gated(temp.path(), Arc::new(Reject));
    assert!(satisfy(&id).run(&mut repo).is_err());
    assert_eq!(lines(temp.path()), before);
}

#[test]
fn a_refused_undo_writes_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let id = seed(temp.path());
    let before = lines(temp.path());
    let mut repo = gated(temp.path(), Arc::new(Reject));
    let refused = Undo {
        reason: "mistake".into(),
    }
    .run(&mut repo);
    assert!(refused.is_err());
    assert_eq!(lines(temp.path()), before);
    assert!(open(temp.path()).get(&id).unwrap().is_some());
}

#[test]
fn the_readings_never_call_the_gate() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    let id = seed(dir);
    let calls = Arc::new(AtomicUsize::new(0));
    let text = id.to_string();

    let repo = gated(dir, Arc::new(Count(calls.clone())));
    let _ = show(&repo, std::slice::from_ref(&text), false).unwrap();
    let _ = list(&repo, &Filter::default()).unwrap();
    let _ = publish(&repo, None).unwrap();

    let as_of = file::open_at(dir, 1, |_| Some("piko".into())).unwrap();
    let as_of = Repository::with_gate(as_of, Arc::new(Count(calls.clone())));
    let _ = show(&as_of, std::slice::from_ref(&text), false).unwrap();

    let reopened = gated(dir, Arc::new(Count(calls.clone())));
    let _ = show(&reopened, std::slice::from_ref(&text), false).unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
