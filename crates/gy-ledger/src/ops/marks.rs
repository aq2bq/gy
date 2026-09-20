//! The one check for a relation's mark: a required relation names a passage
//! that must appear in the older decision's body or applicability conditions
//! (D-75: reject, do not warn).
use crate::model::{Node, NodeData, Relation};
use crate::store::{Error, Result};

/// Relations whose mark names the passage that loses effect.
pub fn required(relation: Relation) -> bool {
    matches!(relation, Relation::Narrows | Relation::Supersedes)
}

/// Whether `mark` names a passage still present in the older decision's body or
/// applicability conditions. The one definition `check` and `edit` share.
pub fn resolves(older: &Node, mark: &str) -> bool {
    let mark = mark.trim();
    if mark.is_empty() {
        return false;
    }
    let scope = match older.data() {
        NodeData::Decision(decision) => decision.scope.text(),
        _ => "",
    };
    older.body().contains(mark) || scope.contains(mark)
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
    if resolves(older, mark) {
        Ok(())
    } else {
        Err(Error::invalid(format!(
            "the mark {mark:?} is not in the older decision's body or applicability conditions"
        )))
    }
}
