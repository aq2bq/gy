//! Retraction: the later decisions that narrow a passage of this one, or
//! replace it whole (d-bde9, n-0c6f). The judgement lives here once; `show`
//! and serve only display it, and never derive it for themselves.
use crate::model::{Node, NodeData, Relation};
use serde::Serialize;
use std::collections::BTreeMap;

/// A passage a later decision narrows, and the decision that narrows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Narrowed {
    pub by: String,
    pub mark: String,
}

/// The retractions a node carries: decisions that replace it whole
/// (`supersedes`) and the passages later decisions narrow (`narrows`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Retraction {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub superseded_by: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub narrowed: Vec<Narrowed>,
}

/// Every node's retractions, from one pass over the graph (n-0c6f). The edges
/// that carry them point from the retracting decision to the older one, so the
/// target is the node that loses effect and the source is the id to name.
pub fn retractions_by_node(all: &[Node]) -> BTreeMap<String, Retraction> {
    let mut out: BTreeMap<String, Retraction> = BTreeMap::new();
    for node in all {
        for edge in node.links() {
            let by = node.id().to_string();
            match edge.label {
                Relation::Supersedes => {
                    out.entry(edge.to.to_string())
                        .or_default()
                        .superseded_by
                        .push(by);
                }
                Relation::Narrows => {
                    if let Some(mark) = &edge.mark {
                        out.entry(edge.to.to_string())
                            .or_default()
                            .narrowed
                            .push(Narrowed {
                                by,
                                mark: mark.clone(),
                            });
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// `text` with each narrowed passage wrapped where it stands, so a reader meets
/// the retraction without looking anywhere else. The passage stays readable
/// inside; the words are English, like the rest of the delivered assets
/// (d-e999). `None` when no passage of `text` loses effect.
pub fn retracted(text: &str, retraction: &Retraction) -> Option<String> {
    let mut marks: Vec<&Narrowed> = retraction
        .narrowed
        .iter()
        .filter(|narrow| !narrow.mark.is_empty())
        .collect();
    if marks.is_empty() {
        return None;
    }
    // Longest first, so a passage that contains another still wins where it
    // starts (the walk only matches at the head of what is left).
    marks.sort_by_key(|narrow| std::cmp::Reverse(narrow.mark.len()));
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    'scan: while !rest.is_empty() {
        for narrow in &marks {
            if rest.starts_with(&narrow.mark) {
                out.push_str(&format!("[[retracted by {}: {}]]", narrow.by, narrow.mark));
                rest = &rest[narrow.mark.len()..];
                continue 'scan;
            }
        }
        let ch = rest.chars().next().expect("rest is not empty");
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    (out != text).then_some(out)
}

/// A decision's applicability conditions with its narrowed passages marked,
/// or `None` when the node is not a decision or none of its scope loses effect.
pub(crate) fn scope_marked(node: &Node, retraction: Option<&Retraction>) -> Option<String> {
    let retraction = retraction?;
    match node.data() {
        NodeData::Decision(data) => retracted(data.scope.text(), retraction),
        _ => None,
    }
}
