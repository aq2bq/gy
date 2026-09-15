//! A small cache for the graph layouts (n-796b), keyed by the point in the log
//! and the scope; only the newest handful is kept.
use crate::layout::Graph;
use std::sync::{Arc, Mutex};

const KEEP: usize = 8;

/// The key: the last write's sequence and the scope (`None` is every scope).
pub type Key = (u64, Option<String>);

/// One remembered layout.
type Entry = (Key, Arc<Graph>);

/// The layouts already computed; cloning shares one cache.
#[derive(Clone, Default)]
pub struct GraphCache {
    entries: Arc<Mutex<Vec<Entry>>>,
}

impl GraphCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// The layout for a key, when it is still here.
    pub fn get(&self, key: &Key) -> Option<Arc<Graph>> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .find(|(held, _)| held == key)
            .map(|(_, graph)| Arc::clone(graph))
    }

    /// Remember a layout, dropping the oldest when the cache is full.
    pub fn put(&self, key: Key, graph: Arc<Graph>) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|(held, _)| held != &key);
        entries.push((key, graph));
        if entries.len() > KEEP {
            entries.remove(0);
        }
    }
}
