//! publish: the human-facing reading that answers the master's six questions
//! (D-85, D-71). This step builds the judgement, undecided, and attention
//! sections; the node and change sections arrive with N-71.
mod waiting;

use crate::ops::repository::{Repository, Result, Store};

/// The Markdown reading for `scope`, with a change section when `since` is
/// given.
pub fn publish<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
    _since: Option<u64>,
) -> Result<String> {
    let mut out = String::from("# gy の公開物\n");
    out.push_str("\n## いま判断待ち\n");
    out.push_str(&waiting::judgement(repository, scope)?);
    out.push_str("\n## 未決の論点\n");
    out.push_str(&waiting::undecided(repository, scope)?);
    out.push_str("\n## 注意\n");
    out.push_str(&waiting::attention(repository, scope)?);
    Ok(out)
}
