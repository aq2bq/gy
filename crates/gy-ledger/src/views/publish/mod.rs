//! publish: the human-facing reading (D-85, D-87). The default is a one-page
//! reading — what waits on the master, the undecided questions, the attention
//! counts, and the period's changes. Naming nodes adds their descriptions.
mod changes;
mod nodes;
mod waiting;

use crate::model::{Node, NodeData, NodeId};
use crate::ops::repository::{Repository, Result, Store};

/// The default one-page reading. The node descriptions are not included; name
/// nodes with `describe` to get them.
pub fn publish<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
    since: Option<u64>,
) -> Result<String> {
    let mut out = String::from("# gy の公開物\n");
    out.push_str("\n## いま判断待ち\n");
    out.push_str(&waiting::judgement(repository, scope)?);
    out.push_str("\n## 未決の論点\n");
    out.push_str(&waiting::undecided(repository, scope)?);
    out.push_str("\n## 注意\n");
    out.push_str(&waiting::attention(repository, scope)?);
    if let Some(since) = since {
        out.push_str("\n## この期間の変更\n");
        out.push_str(&changes::changes(repository, since)?);
    }
    Ok(out)
}

/// The description of the named nodes only, with nothing else (D-87).
pub fn describe<S: Store>(repository: &Repository<S>, ids: &[NodeId]) -> Result<String> {
    nodes::describe(repository, ids)
}

/// The new id with its old aliases and, for a requirement, its reference beside
/// it.
pub(super) fn label(node: &Node) -> String {
    let mut extra: Vec<String> = node.aliases().iter().map(|alias| alias.0.clone()).collect();
    if let NodeData::Requirement(data) = node.data() {
        if let Some(reference) = &data.reference {
            extra.push(reference.0.clone());
        }
    }
    if extra.is_empty() {
        node.id().to_string()
    } else {
        format!("{} ({})", node.id(), extra.join(", "))
    }
}

pub(super) fn in_scope(node: &Node, scope: Option<&str>) -> bool {
    scope.is_none_or(|scope| node.scope() == scope)
}
