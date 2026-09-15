//! The one check for a relation's mark: a required relation names a passage
//! that must appear in the older decision's body or applicability conditions
//! (D-75: reject, do not warn).
use crate::model::{Node, NodeData, Relation};
use crate::store::{Error, Result};

/// Relations whose mark names the passage that loses effect.
pub fn required(relation: Relation) -> bool {
    matches!(relation, Relation::Narrows | Relation::Supersedes)
}

pub fn check(older: &Node, mark: Option<&str>, required: bool) -> Result<()> {
    let Some(mark) = mark.map(str::trim).filter(|mark| !mark.is_empty()) else {
        return if required {
            Err(Error::invalid(
                "this relation needs a mark naming the affected passage",
            ))
        } else {
            Ok(())
        };
    };
    let scope = match older.data() {
        NodeData::Decision(decision) => decision.scope.text(),
        _ => "",
    };
    if older.body().contains(mark) || scope.contains(mark) {
        Ok(())
    } else {
        Err(Error::invalid(format!(
            "the mark {mark:?} is not in the older decision's body or applicability conditions"
        )))
    }
}
