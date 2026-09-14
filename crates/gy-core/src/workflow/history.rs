//! Validation of immutable record and transition snapshots.
use super::{comparison::check_records, schema::validate_object};
use crate::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn snapshots(node: &Node) -> impl Iterator<Item = &Value> {
    node.attrs
        .get("record_history")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(
            node.attrs
                .get("transitions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|t| t.get("workflow")),
        )
}

pub(super) fn history_issues(node: &Node) -> Vec<String> {
    let mut issues = vec![];
    for name in ["record_history", "transitions"] {
        if node.attrs.get(name).is_some_and(|v| !v.is_array()) {
            issues.push(format!(
                "{name} must be an array; historical records cannot be replaced"
            ));
        }
    }
    for snapshot in snapshots(node) {
        if !snapshot.get("records").is_some_and(Value::is_object)
            || !snapshot.get("schemas").is_some_and(Value::is_object)
            || !snapshot.get("checks").is_some_and(Value::is_array)
            || snapshot
                .get("evidence")
                .and_then(Value::as_str)
                .is_none_or(|s| s.trim().is_empty())
            || snapshot
                .get("at")
                .and_then(Value::as_str)
                .is_none_or(|s| chrono::DateTime::parse_from_rfc3339(s).is_err())
            || snapshot.get("waived").is_some_and(|waived| {
                !waived
                    .as_object()
                    .is_some_and(|m| m.values().all(Value::is_string))
            })
        {
            issues.push(
                "Malformed workflow history snapshot; preserve the original recorded inputs".into(),
            );
            continue;
        }
        // Historical inputs are checked against the schema saved with them,
        // never against today's project configuration.
        let schemas =
            serde_json::from_value::<BTreeMap<String, RecordSchema>>(snapshot["schemas"].clone());
        let checks = serde_json::from_value::<Vec<RecordCheck>>(snapshot["checks"].clone());
        let (Ok(schemas), Ok(checks)) = (schemas, checks) else {
            issues.push("Malformed workflow history schemas or checks".into());
            continue;
        };
        let config = WorkflowConfig {
            records: schemas,
            guards: BTreeMap::new(),
        };
        if let Err(error) = config.validate() {
            issues.push(format!("Invalid workflow history schema: {error}"));
            continue;
        }
        let records = snapshot["records"].as_object().unwrap();
        if records.keys().collect::<BTreeSet<_>>() != config.records.keys().collect::<BTreeSet<_>>()
        {
            issues.push("Workflow history records must match their saved schemas".into());
            continue;
        }
        for (name, schema) in &config.records {
            validate_object(
                &records[name],
                &schema.fields,
                &format!("history.{name}"),
                &mut issues,
            );
        }
        let historical = Node {
            attrs: records.clone(),
            body: String::new(),
            path: Default::default(),
        };
        issues.extend(
            check_records(&historical, &checks)
                .into_iter()
                .map(|s| format!("history: {s}")),
        );
    }
    issues
}
