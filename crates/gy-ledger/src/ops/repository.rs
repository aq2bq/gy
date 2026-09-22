//! Typed node access over a store: get, all, put, remove, id resolution, and
//! one intent per transaction (D-69, D-75).
use crate::model::{Edge, Node, NodeId, NodeKind};
// Re-exported so a view can use `Repository` without reaching into the store
// layer (D-76): the trait bound and its result are part of this API.
pub(crate) use super::snapshot::Snapshot;
use super::snapshot::{incoming_in, resolve_in};
use crate::store::Gate;
pub use crate::store::{Error, Result, Store, SyncStatus};
use std::sync::Arc;

pub struct Repository<S: Store> {
    store: S,
    why: String,
    source: String,
    scopes: Vec<String>,
    retries: u32,
}
impl<S: Store> Repository<S> {
    /// The repository with this build's rule installed as the gate (n-557f).
    pub fn new(store: S) -> Self {
        Self::with_gate(store, Arc::new(super::judge::Rules))
    }
    /// The repository with a chosen gate, for tests (n-557f).
    #[doc(hidden)]
    pub fn with_gate(mut store: S, gate: Arc<dyn Gate>) -> Self {
        store.set_gate(gate);
        Self {
            store,
            why: String::new(),
            source: String::new(),
            scopes: Vec::new(),
            retries: 0,
        }
    }
    /// The scope names a move accepts, from gy.toml. An empty list accepts
    /// none, so a node can never be moved into an undeclared scope (n-1aad).
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }
    /// The scope names this repository accepts for a move.
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
    /// The store beneath this repository.
    pub fn store(&self) -> &S {
        &self.store
    }
    /// The retries the next write carries into its log line (n-fe59).
    pub fn retries(&self) -> u32 {
        self.retries
    }
    /// Set the retries the next write carries. The store learns it at once, so
    /// an operation that commits outside `transaction` (undo) still carries it.
    pub fn set_retries(&mut self, retries: u32) {
        self.retries = retries;
        self.store.set_retries(retries);
    }
    /// The store beneath this repository, for operations that own their own
    /// transaction such as undo.
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }
    pub fn next_id(&mut self, kind: NodeKind) -> Result<NodeId> {
        NodeId::mint(kind, &mut self.store)
    }
    pub fn get(&self, id: &NodeId) -> Result<Option<Node>> {
        self.load(&id.to_string())
    }
    pub fn all(&self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        for key in self.store.keys() {
            if let Some(node) = self.load(&key)? {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }
    /// The reverse of every edge that points at `id`, derived from the from
    /// side's stored links (D-76: one place per edge).
    pub fn incoming(&self, id: &NodeId) -> Result<Vec<Edge>> {
        Ok(incoming_in(&self.all()?, id))
    }
    /// Every node, its id index, and its reverse edges, from one pass over the
    /// store (n-2e03). A caller that reads many nodes takes this once instead of
    /// scanning per node.
    pub(crate) fn snapshot(&self) -> Result<Snapshot> {
        Ok(super::snapshot::build(self.all()?))
    }
    pub fn put(&mut self, node: &Node) -> Result<()> {
        let bytes = encode(node)?;
        let id = node.id().to_string();
        let what = self.store.what_for(&id, false);
        self.store.stage(id.clone(), bytes);
        self.store.record(&id, what, &self.why, &self.source);
        Ok(())
    }
    pub fn remove(&mut self, id: &NodeId) -> Result<()> {
        let key = id.to_string();
        self.store.stage(key.clone(), Vec::new());
        self.store.record(&key, "deleted", &self.why, &self.source);
        Ok(())
    }
    /// Rename a scope in every node that carries it, as one change (n-ff2b).
    /// The caller rewrites gy.toml after the transaction commits.
    pub fn rename_scope(&mut self, from: &str, to: &str) -> Result<usize> {
        let nodes = self.store.rename_scope(from, to)?;
        self.store.record(
            "",
            &crate::store::rename_line(from, to, nodes),
            &self.why,
            &self.source,
        );
        Ok(nodes)
    }
    /// Resolve an exact id, an alias (a zero-padded old id, so `D-6` = `D-06`),
    /// or a requirement's outward reference by exact or suffix match. An exact
    /// id wins, and the zero-padding is only among aliases, never a hash id. An
    /// unknown text is an error, and an ambiguous one lists the candidates.
    pub fn resolve(&self, text: &str) -> Result<NodeId> {
        resolve_in(&self.all()?, text)
    }
    /// Run `f` as one transaction with the given why and source. A failed
    /// commit rolls back, so nothing is written (AC-45).
    pub fn transaction<T>(
        &mut self,
        why: &str,
        source: &str,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.why = why.to_string();
        self.source = source.to_string();
        self.store.set_retries(self.retries);
        self.store.begin();
        match f(self) {
            Ok(value) => match self.store.commit() {
                Ok(()) => Ok(value),
                Err(error) => {
                    self.store.rollback();
                    Err(error)
                }
            },
            Err(error) => {
                self.store.rollback();
                Err(error)
            }
        }
    }
    fn load(&self, key: &str) -> Result<Option<Node>> {
        match self.store.get(key) {
            Some(bytes) => Ok(Some(
                serde_json::from_slice(&bytes).map_err(|e| Error::invalid(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }
}

fn encode(node: &Node) -> Result<Vec<u8>> {
    serde_json::to_vec(node).map_err(|error| Error::invalid(error.to_string()))
}
