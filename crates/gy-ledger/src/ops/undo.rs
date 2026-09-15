//! undo: invert the last transaction as a new transaction, with a reason.
use super::{Operation, Outcome, Repository};
use crate::store::{Error, Result, Store, UNDO_WHY, UndoneKind};

pub struct Undo {
    pub reason: String,
}
impl<S: Store> Operation<S> for Undo {
    type Output = ();
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.reason.trim().is_empty() {
            return Err(Error::invalid("undo needs a reason"));
        }
        let next = match repo.store_mut().undo(UNDO_WHY, &self.reason)? {
            UndoneKind::Undo { seq } => vec![format!("redo になった（{seq} の undo を戻した）")],
            UndoneKind::Write => vec!["もう一度 undo すると今の undo を戻す（redo）".to_string()],
        };
        Ok(Outcome {
            id: None,
            changed: vec!["undone".into()],
            missing: Vec::new(),
            next,
            value: (),
        })
    }
}
