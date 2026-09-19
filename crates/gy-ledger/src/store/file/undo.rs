//! Undoing the file store's last transaction, kept out of the transaction
//! machinery so each file stays small (n-38e3).
use super::super::{Error, Result, Store, UndoneKind, log, replay, undone_kind};
use super::FileStore;
use std::collections::BTreeMap;

impl FileStore {
    /// Append the inverse of the last transaction as a new line (D-82). The
    /// log is never rewritten; the undo is one more transaction with the given
    /// why and source. A created node is deleted, an updated node returns to
    /// its earlier value, and a deleted node comes back.
    pub(super) fn undo_impl(&mut self, why: &str, source: &str) -> Result<UndoneKind> {
        let (events, _) = log::read(&self.dir)?;
        let last = events
            .last()
            .ok_or_else(|| Error::invalid("there is nothing to undo"))?;
        self.own_write(last)?;
        let kind = undone_kind(&last.why, last.seq);
        let mut before = BTreeMap::new();
        for event in &events[..events.len() - 1] {
            replay::apply(&mut before, event);
        }
        self.begin();
        for change in &last.changes {
            match change {
                log::Change::Created { node, .. } => {
                    self.stage(node.clone(), Vec::new());
                    self.record(node, "created", why, source);
                }
                log::Change::Updated { node, .. } => {
                    let value =
                        replay::encode(before.get(node).ok_or_else(|| {
                            Error::invalid("the updated node has no earlier value")
                        })?)?;
                    self.stage(node.clone(), value);
                    self.record(node, "updated", why, source);
                }
                log::Change::Deleted { node, value } => {
                    self.stage(node.clone(), replay::encode(value)?);
                    self.record(node, "deleted", why, source);
                }
                log::Change::ScopeRenamed { from, to, .. } => {
                    let nodes = self.rename_scope(to, from)?;
                    self.record("", &super::super::rename_line(to, from, nodes), why, source);
                }
            }
        }
        self.commit()?;
        Ok(kind)
    }
}
