//! publish's scan over the store (n-2e03, ac-f4a4): one pass, not one per
//! node. `Store::keys` is the full scan `all` and `resolve` both take, so
//! counting its calls counts the scans.
use gy_ledger::{
    Actor, FormatVersion, Gate, HistoryEntry, MemoryStore, Node, NodeId, NodeKind, Repository,
    Result, Store, UndoneKind, publish,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const SCOPE: &str = "a";
const DATE: &str = "2026-09-15T00:00:00Z";

/// A store that counts every full scan of its keys, delegating the rest.
struct Counting<S: Store> {
    inner: S,
    scans: Arc<AtomicUsize>,
}
impl<S: Store> gy_ledger::IdSource for Counting<S> {
    fn next_hash(&mut self, prefix: &str) -> Result<String> {
        self.inner.next_hash(prefix)
    }
}
impl<S: Store> Store for Counting<S> {
    fn version(&self) -> FormatVersion {
        self.inner.version()
    }
    fn set_retries(&mut self, retries: u32) {
        self.inner.set_retries(retries);
    }
    fn set_gate(&mut self, gate: Arc<dyn Gate>) {
        self.inner.set_gate(gate);
    }
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.get(key)
    }
    fn keys(&self) -> Vec<String> {
        self.scans.fetch_add(1, Ordering::SeqCst);
        self.inner.keys()
    }
    fn begin(&mut self) {
        self.inner.begin();
    }
    fn stage(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.inner.stage(key, value);
    }
    fn rename_scope(&mut self, from: &str, to: &str) -> Result<usize> {
        self.inner.rename_scope(from, to)
    }
    fn commit(&mut self) -> Result<()> {
        self.inner.commit()
    }
    fn rollback(&mut self) {
        self.inner.rollback();
    }
    fn record(&mut self, node: &str, what: &str, why: &str, source: &str) {
        self.inner.record(node, what, why, source);
    }
    fn history(&self) -> &[HistoryEntry] {
        self.inner.history()
    }
    fn undo(&mut self, why: &str, source: &str) -> Result<UndoneKind> {
        self.inner.undo(why, source)
    }
}

/// The full scans one `publish` of `n` nodes takes.
fn scans(n: usize) -> usize {
    let scans = Arc::new(AtomicUsize::new(0));
    let store = MemoryStore::with_actor(FormatVersion::CURRENT, Actor::new("piko").unwrap());
    let mut repo = Repository::new(Counting {
        inner: store,
        scans: scans.clone(),
    });
    let nodes: Vec<Node> = (0..n)
        .map(|i| {
            Node::criterion(
                NodeId::from_hash(NodeKind::Criterion, format!("{i:04}")).unwrap(),
                SCOPE,
                DATE,
                "a criterion",
            )
            .unwrap()
        })
        .collect();
    repo.transaction("seed", "test", |repo| {
        for node in &nodes {
            repo.put(node)?;
        }
        Ok(())
    })
    .unwrap();

    scans.store(0, Ordering::SeqCst);
    publish(&repo, None, None, "", "").unwrap();
    scans.load(Ordering::SeqCst)
}

#[test]
fn publish_scans_the_store_once_however_many_nodes() {
    assert_eq!(scans(20), 1, "publish takes one full scan");
    assert_eq!(
        scans(2),
        1,
        "the scan count does not depend on the node count"
    );
}
