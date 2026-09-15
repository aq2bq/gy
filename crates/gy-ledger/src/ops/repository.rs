//! Typed node access over a store: get, all, put, remove, id resolution, and
//! one intent per transaction (D-69, D-75).
use crate::model::{Edge, Node, NodeData, NodeId, NodeKind};
// Re-exported so a view can use `Repository` without reaching into the store
// layer (D-76): the trait bound and its result are part of this API.
pub use crate::store::{Error, Result, Store};

pub struct Repository<S: Store> {
    store: S,
    why: String,
    source: String,
}
impl<S: Store> Repository<S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            why: String::new(),
            source: String::new(),
        }
    }
    /// The store beneath this repository.
    pub fn store(&self) -> &S {
        &self.store
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
        let mut edges = Vec::new();
        for node in self.all()? {
            for edge in node.links() {
                if &edge.to == id {
                    edges.push(Edge {
                        from: id.clone(),
                        label: edge.label,
                        reversed: true,
                        mark: edge.mark.clone(),
                        to: edge.from.clone(),
                    });
                }
            }
        }
        Ok(edges)
    }
    pub fn put(&mut self, node: &Node) -> Result<()> {
        let bytes = encode(node)?;
        let id = node.id().to_string();
        self.store.stage(id.clone(), bytes);
        self.store.record(&id, "put", &self.why, &self.source);
        Ok(())
    }
    pub fn remove(&mut self, id: &NodeId) -> Result<()> {
        let key = id.to_string();
        self.store.stage(key.clone(), Vec::new());
        self.store.record(&key, "deleted", &self.why, &self.source);
        Ok(())
    }
    /// Resolve an exact id, an alias (a zero-padded old id, so `D-6` = `D-06`),
    /// or a requirement's outward reference by exact or suffix match. An exact
    /// id wins, and the zero-padding is only among aliases, never a hash id. An
    /// unknown text is an error, and an ambiguous one lists the candidates.
    pub fn resolve(&self, text: &str) -> Result<NodeId> {
        let wanted = text.to_lowercase();
        let old = normalize(text);
        let (mut by_id, mut by_alias, mut by_ref) = (Vec::new(), Vec::new(), Vec::new());
        for key in self.store.keys() {
            let Some(node) = self.load(&key)? else {
                continue;
            };
            if key.to_lowercase() == wanted {
                by_id.push(node.id().clone());
            } else if node
                .aliases()
                .iter()
                .any(|alias| normalize(&alias.0) == old)
            {
                by_alias.push(node.id().clone());
            } else if referenced(&node).is_some_and(|ref_| ref_ == text || ref_.ends_with(text)) {
                by_ref.push(node.id().clone());
            }
        }
        if by_id.is_empty() {
            choose(
                text,
                if by_alias.is_empty() {
                    by_ref
                } else {
                    by_alias
                },
            )
        } else {
            choose(text, by_id)
        }
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

fn names(matches: &[NodeId]) -> String {
    matches
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A requirement's outward reference, if any.
fn referenced(node: &Node) -> Option<&str> {
    match node.data() {
        NodeData::Requirement(requirement) => requirement.reference.as_ref().map(|r| r.0.as_str()),
        _ => None,
    }
}

fn choose(text: &str, mut matches: Vec<NodeId>) -> Result<NodeId> {
    match matches.len() {
        0 => Err(Error::invalid(format!("no node matches {text}"))),
        1 => Ok(matches.remove(0)),
        _ => Err(Error::invalid(format!(
            "{text} matches several nodes: {}",
            names(&matches)
        ))),
    }
}

/// Lowercase the kind and drop leading zeros from an all-digit suffix, so
/// `D-06` and `d-6` name the same old id.
fn normalize(text: &str) -> String {
    match text.rsplit_once('-') {
        Some((kind, suffix)) if suffix.bytes().all(|byte| byte.is_ascii_digit()) => {
            let digits = suffix.trim_start_matches('0');
            format!(
                "{}-{}",
                kind.to_lowercase(),
                if digits.is_empty() { "0" } else { digits }
            )
        }
        _ => text.to_lowercase(),
    }
}
