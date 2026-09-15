//! publish: the human-facing reading that answers the master's six questions
//! (D-85, D-71). Every id is printed beside its title (D-62).
mod changes;
mod nodes;
mod waiting;

use crate::model::{Node, NodeData};
use crate::ops::repository::{Repository, Result, Store};

/// The Markdown reading for `scope`, with a change section when `since` is
/// given.
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
    out.push_str("\n## ノード\n");
    out.push_str(&nodes::nodes(repository, scope)?);
    if let Some(since) = since {
        out.push_str("\n## この期間の変更\n");
        out.push_str(&changes::changes(repository, since)?);
    }
    out.push_str("\n## 注意\n");
    out.push_str(&waiting::attention(repository, scope)?);
    Ok(out)
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
