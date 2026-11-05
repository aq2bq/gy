//! Retire an explicitly selected set as one reversible transaction.
use super::{Operation, Outcome, Repository, snapshot::resolve_in};
use crate::model::NodeId;
use crate::store::{Error, Result, Store};

#[derive(Debug, Clone)]
pub struct Retire {
    pub ids: Vec<String>,
    pub reason: String,
    pub revoke: bool,
}
impl<S: Store> Operation<S> for Retire {
    type Output = Vec<NodeId>;
    fn run(self, repo: &mut Repository<S>) -> Result<Outcome<Self::Output>> {
        if self.reason.trim().is_empty() {
            return Err(Error::invalid("retire needs a reason"));
        }
        let all = repo.all_stored()?;
        let mut ids = self
            .ids
            .iter()
            .map(|text| resolve_in(&all, text))
            .collect::<Result<Vec<_>>>()?;
        ids.sort_by_key(ToString::to_string);
        ids.dedup();
        let names: Vec<String> = ids.iter().map(ToString::to_string).collect();
        let action = if self.revoke {
            "retirement-revoked"
        } else {
            "retired"
        };
        repo.transaction("retire", &self.reason, |repo| {
            repo.store_mut().retire(&names, self.revoke)?;
            repo.store_mut().record(
                "",
                &format!("{action} {}", names.join(" ")),
                "retire",
                &self.reason,
            );
            Ok(())
        })?;
        Ok(Outcome {
            id: None,
            changed: vec![
                format!("{action} {}", names.join(" ")),
                "元の文はログと remote の git に残る".into(),
            ],
            missing: Vec::new(),
            next: Vec::new(),
            unresolved: Vec::new(),
            value: ids,
        })
    }
}
