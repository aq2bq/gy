use crate::*;
use serde::Serialize;
use serde_json::{Value, json};

pub const EDGES: &[(&str, &str, &str, &str)] = &[
    ("closes", "question", "decision", "closes"),
    ("narrows", "decision", "decision", "narrowed-by"),
    ("widens", "decision", "decision", "widened-by"),
    ("supersedes", "decision", "decision", "superseded-by"),
    ("completes", "decision", "decision", "completed-by"),
    ("targets", "need|requirement", "criterion", "targeted-by"),
    ("spawned-by", "need", "decision", "spawns"),
    ("filed-as", "need", "requirement", "filed-from"),
    ("depends-on", "need", "need", "needed-by"),
    ("relies-on", "requirement", "decision", "relied-on-by"),
    ("raised", "requirement", "question", "raised-by"),
    ("measured-by", "gate", "question", "measures"),
];
pub const REFERENCE_KEYS: &[&str] = &["waiting-on", "unresolved", "belongs-to"];

fn insert_edge(node: &mut Node, key: &str, id: &str, mark: Option<&str>) {
    let mut values = match node.attrs.get(key) {
        Some(Value::Array(a)) => a.clone(),
        Some(v) => vec![v.clone()],
        None => vec![],
    };
    values.retain(|v| !ids(Some(v)).iter().any(|s| s == id));
    values.push(match mark {
        Some(mark) => json!({"id":id,"mark":mark}),
        None => json!(id),
    });
    node.put(key, values);
}
pub fn edge_mark(node: &Node, key: &str, id: &str) -> Option<String> {
    let value = node.attrs.get(key)?;
    let items = match value {
        Value::Array(a) => a.iter().collect(),
        v => vec![v],
    };
    items
        .into_iter()
        .find(|v| v.get("id").and_then(Value::as_str) == Some(id))
        .and_then(|v| v.get("mark"))
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
}
impl Store {
    pub fn link(
        &mut self,
        source: &str,
        label: &str,
        target: &str,
        mark: Option<&str>,
    ) -> Result<Vec<String>> {
        let edge = EDGES.iter().find(|e| e.0 == label).ok_or_else(|| {
            Error::input(format!(
                "Unknown relationship {label}. Check directions with gy link --help"
            ))
        })?;
        self.typed(source, edge.1)?;
        self.typed(target, edge.2)?;
        if source == target {
            return Err(Error::input("Cannot create a link to the same node"));
        }
        if label == "depends-on"
            && self.reaches(
                target,
                source,
                label,
                &mut std::collections::BTreeSet::new(),
            )
        {
            return Err(Error::input(
                "This creates a depends-on cycle, preventing prerequisites from being resolved",
            ));
        }
        if label == "closes" {
            let q = self.node(source)?;
            if q.closed() && !q.refs("closes").contains(&target.to_string()) {
                return Err(Error::input(format!("{source} is already closed")));
            }
        }
        insert_edge(self.nodes.get_mut(source).unwrap(), label, target, mark);
        insert_edge(self.nodes.get_mut(target).unwrap(), edge.3, source, mark);
        let mut warnings = vec![];
        if ["narrows", "supersedes"].contains(&label) && mark.is_none_or(|m| m.trim().is_empty()) {
            warnings.push("--mark is missing. A file-level link cannot identify which passage in the older decision is invalidated".into());
        }
        if let Some(mark) = mark {
            if !self.node(target)?.body.contains(mark) {
                warnings.push(format!("The anchor text is missing from the body of {target}. show / render will describe the affected passage separately: {mark}"));
            }
        }
        Ok(warnings)
    }
    pub fn reaches(
        &self,
        from: &str,
        to: &str,
        key: &str,
        seen: &mut std::collections::BTreeSet<String>,
    ) -> bool {
        if from == to {
            return true;
        }
        if !seen.insert(from.into()) {
            return false;
        }
        self.nodes
            .get(from)
            .is_some_and(|n| n.refs(key).iter().any(|id| self.reaches(id, to, key, seen)))
    }
    pub fn waiting_references(&self, question: &str) -> Vec<String> {
        self.nodes
            .values()
            .filter(|n| {
                n.id() != question
                    && (n
                        .refs("waiting-on")
                        .iter()
                        .chain(n.refs("unresolved").iter())
                        .any(|id| id == question)
                        || n.body.lines().any(|line| {
                            contains_id(line, question)
                                && ["undecided", "unresolved", "waiting"]
                                    .iter()
                                    .any(|word| line.contains(word))
                        }))
            })
            .map(|n| n.id().into())
            .collect()
    }
}
pub fn contains_id(s: &str, id: &str) -> bool {
    s.match_indices(id).any(|(start, _)| {
        let before = s[..start].chars().next_back();
        let after = s[start + id.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-')
            && !after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}
#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub rule: String,
    pub severity: String,
    pub id: String,
    pub message: String,
}
impl Store {
    pub fn lint(&self, scope: Option<&str>) -> Vec<Diagnostic> {
        let mut out = vec![];
        let mut emit = |rule: &str, node: &Node, message: String, force: bool| {
            if scope.is_some_and(|s| s != node.scope()) {
                return;
            }
            let default = if rule == "L6" { "warn" } else { "error" };
            let severity = if force {
                Some("error")
            } else {
                self.config
                    .lint
                    .get(rule)
                    .map(|r| r.severity().unwrap())
                    .unwrap_or(Some(default))
            };
            if let Some(severity) = severity {
                out.push(Diagnostic {
                    rule: rule.into(),
                    severity: severity.into(),
                    id: node.id().into(),
                    message,
                });
            }
        };
        for n in self.nodes.values() {
            if n.closed() {
                for id in self.waiting_references(n.id()) {
                    if let Some(referrer) = self.nodes.get(&id) {
                        emit(
                            "L1",
                            referrer,
                            format!("References closed question {} as unresolved", n.id()),
                            false,
                        );
                    }
                }
            }
            if n.kind() == "criterion" {
                let actual = self
                    .nodes
                    .values()
                    .filter(|m| {
                        m.kind() == "need" && m.refs("targets").contains(&n.id().to_owned())
                    })
                    .count() as u64;
                let stated = n
                    .attrs
                    .get("bearer_count")
                    .and_then(Value::as_u64)
                    .or_else(|| {
                        n.body
                            .split("bearers")
                            .nth(1)
                            .and_then(|s| {
                                s.trim_start().split(|c: char| !c.is_ascii_digit()).next()
                            })
                            .and_then(|s| s.parse().ok())
                    });
                if let Some(stated) = stated {
                    if stated != actual {
                        emit(
                            "L2",
                            n,
                            format!(
                                "Stated bearer count is {stated}, but {actual} needs have targets links"
                            ),
                            false,
                        );
                    }
                }
            }
            if n.kind() == "need"
                && !n
                    .refs("targets")
                    .iter()
                    .any(|id| self.nodes.get(id).is_some_and(|x| x.kind() == "criterion"))
            {
                emit("L3", n, "A need without an acceptance criterion disappears from the progress overview. Specify targets".into(), false);
            }
            if n.kind() == "question" {
                let owners: std::collections::BTreeSet<_> = n
                    .refs("belongs-to")
                    .into_iter()
                    .chain(n.refs("raised-by"))
                    .collect();
                if owners.len() > 1 {
                    emit(
                        "L4",
                        n,
                        "The question has more than one owner".into(),
                        false,
                    );
                }
                let capture = n.attrs.get("capture").and_then(Value::as_bool) == Some(true);
                if n.get("decider").trim().is_empty() {
                    emit(
                        "L8",
                        n,
                        "decider is empty. Record whose agreement is required".into(),
                        capture,
                    );
                }
                if n.refs("options")
                    .iter()
                    .filter(|s| !s.trim().is_empty())
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    < 2
                {
                    emit("L9", n, "Record at least two viable options. Something uniquely determined by facts is not a question to decide".into(), capture);
                }
                if !n.get("bundle").is_empty() && n.get("bundle-rationale").trim().is_empty() {
                    emit(
                        "L12",
                        n,
                        "bundle-rationale is missing: explain why the same intervention closes the bundled questions".into(),
                        false,
                    );
                }
            }
            if n.kind() == "decision" {
                if n.get("decision_scope").trim().is_empty() {
                    emit("L7", n, "Applicability conditions are empty. Record the applicable paths and conditions so later users do not apply the decision too broadly".into(), false);
                }
                for key in ["narrows", "supersedes"] {
                    for id in n.refs(key) {
                        if edge_mark(n, key, &id).is_none() {
                            emit(
                                "L6",
                                n,
                                format!(
                                    "{key} {id} has no mark identifying the invalidated passage"
                                ),
                                false,
                            );
                        }
                    }
                }
            }
            if n.kind() == "requirement" {
                if base_state(n.get("status")).is_none() {
                    emit(
                        "L10",
                        n,
                        format!(
                            "State {} is not one of the 11 allowed states",
                            n.get("status")
                        ),
                        false,
                    );
                }
                if base_state(n.get("status")) == Some("complete") {
                    let issues = if n.attrs.contains_key("compressed") {
                        self.compression_field_issues(n)
                    } else {
                        self.completion_issues(n)
                    };
                    for message in issues {
                        emit("L11", n, message, false);
                    }
                }
                for id in n.refs("relies-on") {
                    if self
                        .nodes
                        .get(&id)
                        .is_some_and(|d| !d.refs("superseded-by").is_empty())
                        || self
                            .nodes
                            .values()
                            .any(|d| d.refs("supersedes").contains(&id))
                    {
                        emit(
                            "L5",
                            n,
                            format!("Relies on superseded decision {id}"),
                            false,
                        );
                    }
                }
            }
            for (key, source_kind, target_kind, reverse) in EDGES {
                let relationships: Vec<(&str, &str, &str)> = if *key == "closes" {
                    vec![(
                        key,
                        if n.kind() == "decision" {
                            "question"
                        } else {
                            "decision"
                        },
                        reverse,
                    )]
                } else {
                    vec![(key, target_kind, reverse), (reverse, source_kind, key)]
                };
                for (field, expected, back) in relationships {
                    let expected_source = if *key == "closes" && n.kind() == "decision" {
                        "decision"
                    } else if field == *key {
                        *source_kind
                    } else {
                        *target_kind
                    };
                    if !n.refs(field).is_empty() && !kind_matches(n.kind(), expected_source) {
                        emit(
                            "edges",
                            n,
                            format!("{field} is an attribute of {expected_source}"),
                            false,
                        );
                    }
                    for id in n.refs(field) {
                        match self.nodes.get(&id) {
                            None => emit(
                                "L13",
                                n,
                                format!("{field} references nonexistent node {id}"),
                                false,
                            ),
                            Some(other) => {
                                if !kind_matches(other.kind(), expected) {
                                    emit(
                                        "edges",
                                        n,
                                        format!("{field} target {id} must be a {expected}"),
                                        false,
                                    );
                                }
                                if !other.refs(back).contains(&n.id().into()) {
                                    emit(
                                        "edges",
                                        n,
                                        format!(
                                            "{id} is missing inverse link {} in {back}",
                                            n.id()
                                        ),
                                        false,
                                    );
                                }
                                if edge_mark(n, field, &id) != edge_mark(other, back, n.id()) {
                                    emit("edges", n, format!("mark does not match {id}"), false);
                                }
                            }
                        }
                    }
                }
            }
            for (key, value) in &n.attrs {
                if key == "id"
                    || EDGES.iter().any(|e| key == e.0 || key == e.3)
                    || REFERENCE_KEYS.contains(&key.as_str())
                {
                    continue;
                }
                for id in attribute_ids(value) {
                    if !self.nodes.contains_key(&id) {
                        emit(
                            "L13",
                            n,
                            format!("{key} references nonexistent node {id}"),
                            false,
                        );
                    }
                }
            }
            for key in REFERENCE_KEYS {
                for id in n.refs(key) {
                    if !self.nodes.contains_key(&id) {
                        emit(
                            "L13",
                            n,
                            format!("{key} references nonexistent node {id}"),
                            false,
                        );
                    }
                }
            }
        }
        out
    }
}
pub fn remaining(n: &Node) -> Option<u64> {
    if n.attrs.contains_key("remaining_work") && legacy_remaining(n) != Some(0) {
        return legacy_remaining(n);
    }
    if n.attrs.contains_key("residual") {
        return residual_destinations(n).ok().map(|_| 0);
    }
    legacy_remaining(n)
}

/// Exact ID-valued attributes remain inspectable even when their key is unknown.
pub fn attribute_ids(value: &Value) -> std::collections::BTreeSet<String> {
    match value {
        Value::String(s) if KINDS.iter().any(|k| valid_id(s, k.0)) => {
            [s.clone()].into_iter().collect()
        }
        Value::Array(a) => a.iter().flat_map(attribute_ids).collect(),
        Value::Object(m) => m
            .iter()
            .filter(|(k, _)| k.as_str() != "mark")
            .flat_map(|(_, v)| attribute_ids(v))
            .collect(),
        _ => Default::default(),
    }
}
