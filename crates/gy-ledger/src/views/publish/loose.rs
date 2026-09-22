//! `loose.md` (n-b6c9): every in-scope node that no page expands, so nothing
//! in the scope goes unread. A question nobody waits on, a criterion nothing
//! targets, a requirement filed without a need.
use super::super::show::Shown;
use super::inline;
use super::wiki::Scope;

pub(super) fn loose_page(scope: &Scope) -> Option<String> {
    let mut loose: Vec<&Shown> = scope
        .all_ids()
        .iter()
        .filter_map(|id| scope.node(id))
        .filter(|node| {
            node.scope.as_deref() == Some(scope.name())
                && !super::is_vertex(node.kind)
                && scope.page_of(&node.id).is_none()
        })
        .collect();
    if loose.is_empty() {
        return None;
    }
    loose.sort_by(|a, b| a.created.cmp(&b.created));
    let mut out = String::from(
        "# On their own\n\n[← All of it](README.md)\n\nNodes no need and no decision \
         reaches. A question nobody waits on, a criterion nothing targets, a \
         requirement filed without a need.\n\n",
    );
    for node in loose {
        out += &inline::block(scope, node, "loose.md");
        out.push('\n');
    }
    Some(out)
}
