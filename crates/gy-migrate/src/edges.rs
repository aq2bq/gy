//! Writing the 0.4 edges onto the new nodes (N-66): the from-side keys as edge
//! values, and waits-on from a need's waiting-on or a gate's measured-by.
use crate::legacy::{Legacy, LegacyLink, LegacyNode};
use gy_ledger::{Link, Node, NodeId, Relation};
use std::collections::BTreeMap;
use std::fmt;

/// What the edge pass did and what it could not write.
#[derive(Debug, Default)]
pub struct Edges {
    pub written: usize,
    pub missing: Vec<String>,
    pub unsupported: Vec<String>,
    pub mark_missing: usize,
    pub waits_on: usize,
    pub waits_skipped: Vec<String>,
    pub belongs_kept: usize,
}

pub fn apply(legacy: &Legacy, nodes: &mut [Node], ids: &BTreeMap<String, NodeId>) -> Edges {
    let mut report = Edges::default();
    for (source, node) in legacy.nodes.iter().zip(nodes.iter_mut()) {
        for (name, links) in source.out_edges() {
            let Some(relation) = relation(name) else {
                report
                    .unsupported
                    .extend(links.iter().map(|link| entry(source, name, &link.id)));
                continue;
            };
            for link in links {
                link_edge(legacy, source, node, relation, link, ids, &mut report);
            }
        }
        waits_on(legacy, source, node, ids, &mut report);
    }
    belongs_to(legacy, nodes, ids, &mut report);
    report
}

fn link_edge(
    legacy: &Legacy,
    source: &LegacyNode,
    node: &mut Node,
    relation: Relation,
    link: &LegacyLink,
    ids: &BTreeMap<String, NodeId>,
    report: &mut Edges,
) {
    let Some(to) = ids.get(&link.id) else {
        report
            .missing
            .push(entry(source, relation.name(), &link.id));
        return;
    };
    match Link::new(node.id().clone(), relation, to.clone()) {
        Ok(edge) => {
            if required_mark(relation) && !mark_in_body(legacy, &link.id, link.mark.as_deref()) {
                report.mark_missing += 1;
            }
            node.link(edge.with_mark(link.mark.clone()));
            report.written += 1;
        }
        Err(_) => report
            .unsupported
            .push(entry(source, relation.name(), &link.id)),
    }
}

/// A need waits on the questions and requirements in its `waiting-on`; a gate's
/// `measured-by` becomes the same relationship (D-83, d-63f8).
fn waits_on(
    legacy: &Legacy,
    source: &LegacyNode,
    node: &mut Node,
    ids: &BTreeMap<String, NodeId>,
    report: &mut Edges,
) {
    if source.new_kind() != "need" {
        return;
    }
    let targets = source
        .waiting_on()
        .iter()
        .map(String::as_str)
        .chain(source.measured_by().iter().map(|link| link.id.as_str()));
    for id in targets {
        let target = ids.get(id).filter(|_| {
            legacy
                .by_id(id)
                .is_some_and(|node| matches!(node.new_kind(), "question" | "requirement"))
        });
        let edge =
            target.and_then(|to| Link::new(node.id().clone(), Relation::WaitsOn, to.clone()).ok());
        match edge {
            Some(edge) => {
                node.link(edge);
                report.waits_on += 1;
            }
            None => report.waits_skipped.push(entry(source, "waits-on", id)),
        }
    }
}

/// A question that belongs to a need means that need waits on the question
/// (d-63f8). A target that is not a need stays as a free attribute.
fn belongs_to(
    legacy: &Legacy,
    nodes: &mut [Node],
    ids: &BTreeMap<String, NodeId>,
    report: &mut Edges,
) {
    let index: BTreeMap<&str, usize> = legacy
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.as_str(), index))
        .collect();
    for (question_index, source) in legacy.nodes.iter().enumerate() {
        if source.new_kind() == "question" {
            one_belonging(legacy, nodes, ids, &index, question_index, source, report);
        }
    }
}

fn one_belonging(
    legacy: &Legacy,
    nodes: &mut [Node],
    ids: &BTreeMap<String, NodeId>,
    index: &BTreeMap<&str, usize>,
    question_index: usize,
    source: &LegacyNode,
    report: &mut Edges,
) {
    let question = nodes[question_index].id().clone();
    let mut kept = Vec::new();
    for target in source.belongs_to() {
        let need = index.get(target.as_str()).filter(|_| {
            legacy
                .by_id(target)
                .is_some_and(|node| node.new_kind() == "need")
        });
        match need {
            Some(&need_index) => {
                if let Some(need_id) = ids.get(target) {
                    if let Ok(edge) =
                        Link::new(need_id.clone(), Relation::WaitsOn, question.clone())
                    {
                        nodes[need_index].link(edge);
                        report.waits_on += 1;
                        continue;
                    }
                }
                kept.push(target.clone());
            }
            None => kept.push(target.clone()),
        }
    }
    if !kept.is_empty() {
        report.belongs_kept += kept.len();
        nodes[question_index].set_free("belongs-to", kept.join(", "));
    }
}

fn required_mark(relation: Relation) -> bool {
    matches!(relation, Relation::Narrows | Relation::Supersedes)
}

fn mark_in_body(legacy: &Legacy, id: &str, mark: Option<&str>) -> bool {
    let Some(mark) = mark else {
        return false;
    };
    legacy
        .by_id(id)
        .is_some_and(|node| node.body.contains(mark))
}

fn entry(source: &LegacyNode, relation: &str, target: &str) -> String {
    format!("{} {relation} {target}", source.id)
}

fn relation(name: &str) -> Option<Relation> {
    [
        Relation::Closes,
        Relation::Narrows,
        Relation::Widens,
        Relation::Supersedes,
        Relation::Completes,
        Relation::Targets,
        Relation::SpawnedBy,
        Relation::FiledAs,
        Relation::DependsOn,
        Relation::ReliesOn,
        Relation::Raised,
        Relation::WaitsOn,
    ]
    .into_iter()
    .find(|relation| relation.name() == name)
}

impl fmt::Display for Edges {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "edges written: {}", self.written)?;
        writeln!(f, "edges missing a target: {}", self.missing.len())?;
        writeln!(f, "edges not allowed by kind: {}", self.unsupported.len())?;
        writeln!(f, "marks not in the old body: {}", self.mark_missing)?;
        writeln!(f, "waits-on written: {}", self.waits_on)?;
        writeln!(f, "waits-on skipped: {}", self.waits_skipped.len())?;
        writeln!(
            f,
            "belongs-to kept as free attributes: {}",
            self.belongs_kept
        )?;
        for entry in &self.missing {
            writeln!(f, "  missing: {entry}")?;
        }
        for entry in &self.unsupported {
            writeln!(f, "  not allowed: {entry}")?;
        }
        for entry in &self.waits_skipped {
            writeln!(f, "  waits-on skipped: {entry}")?;
        }
        Ok(())
    }
}
