//! undo: invert the last transaction as a new transaction, with a reason.
use super::{Operation, Outcome, Repository};
use crate::store::{Error, Result, Store};

pub struct Undo {
    pub reason: String,
}
impl<S: Store> Operation<S> for Undo {
    type Output = ();
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.reason.trim().is_empty() {
            return Err(Error::invalid("undo needs a reason"));
        }
        repo.store_mut().undo("undo", &self.reason)?;
        Ok(Outcome {
            id: None,
            changed: vec!["undone".into()],
            missing: Vec::new(),
            next: Vec::new(),
            value: (),
        })
    }
}
