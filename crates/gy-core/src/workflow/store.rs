//! Store operations that validate and snapshot configured records.
use super::{
    comparison::check_records,
    history::{history_issues, snapshots},
    schema::validate_object,
};
use crate::*;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

impl Store {
    fn record_issues(&self, node: &Node, name: &str) -> Vec<String> {
        let mut issues = vec![];
        let Some(schema) = self.config.workflow.records.get(name) else {
            return vec![format!("Unknown workflow record {name}")];
        };
        if !schema.kinds.iter().any(|k| k == node.kind()) {
            return vec![format!("{name} is not a record for {}", node.kind())];
        }
        let Some(value) = node.attrs.get(name) else {
            return vec![format!("{name} is required")];
        };
        validate_object(value, &schema.fields, name, &mut issues);
        if let Some(key) = &schema.version_field {
            if let Some(version) = value.get(key).and_then(Value::as_str) {
                for snapshot in snapshots(node) {
                    if let Some(old) = snapshot.get("records").and_then(|v| v.get(name)) {
                        if old.get(key).and_then(Value::as_str) == Some(version) && old != value {
                            issues.push(format!("{name}.{key}={version} was already recorded with different contents; use a new version"));
                            break;
                        }
                    }
                }
            }
        }
        issues
    }

    /// Passive inspection follows lifecycle: completed work has historical
    /// records, while active work must satisfy today's configured policy.
    pub fn workflow_issues(&self, node: &Node, state: Option<&str>) -> Vec<String> {
        if node.is_complete_requirement() {
            return history_issues(node);
        }
        self.current_workflow_issues(node, state)
    }

    /// A guard is waived only when its waiver record is present and valid
    /// against the configured schema. An absent record leaves the guard in
    /// force without reporting the absence; an invalid one is reported as a
    /// record issue while the guard still applies.
    fn guard_waiver(&self, node: &Node, guard: &StateGuard) -> Option<String> {
        let name = guard.waived_by.as_ref()?;
        if !node.attrs.contains_key(name) {
            return None;
        }
        self.record_issues(node, name)
            .is_empty()
            .then(|| name.clone())
    }

    fn current_workflow_issues(&self, node: &Node, state: Option<&str>) -> Vec<String> {
        let history_issues = history_issues(node);
        let mut names: BTreeSet<String> = self
            .config
            .workflow
            .records
            .iter()
            .filter(|(name, schema)| {
                (schema.kinds.iter().any(|k| k == node.kind()) && schema.required)
                    || node.attrs.contains_key(*name)
            })
            .map(|(name, _)| name.clone())
            .collect();
        let guards: Vec<_> = self
            .config
            .workflow
            .guards
            .values()
            .filter(|guard| {
                node.kind() == "requirement"
                    && state.is_some_and(|s| guard.states.iter().any(|g| g == s))
            })
            .collect();
        let mut checks = vec![];
        for guard in &guards {
            // Include a present waiver record so its own schema defects are
            // reported even when they stop it from waiving the guard.
            if let Some(name) = &guard.waived_by {
                if node.attrs.contains_key(name) {
                    names.insert(name.clone());
                }
            }
            if self.guard_waiver(node, guard).is_some() {
                continue;
            }
            names.extend(guard.records.iter().cloned());
            checks.extend(guard.checks.iter().cloned());
        }
        let mut issues: Vec<_> = history_issues
            .into_iter()
            .chain(names.iter().flat_map(|name| self.record_issues(node, name)))
            .collect();
        issues.extend(check_records(node, &checks));
        issues
    }

    fn workflow_snapshot(
        &self,
        node: &Node,
        names: &[String],
        checks: &[RecordCheck],
        evidence: &str,
        waived: Option<&Value>,
    ) -> Value {
        let records: Map<_, _> = names
            .iter()
            .filter_map(|name| node.attrs.get(name).map(|v| (name.clone(), v.clone())))
            .collect();
        let schemas: BTreeMap<_, _> = names
            .iter()
            .map(|name| (name, &self.config.workflow.records[name]))
            .collect();
        let mut snapshot = json!({"at": today(), "evidence": evidence, "records": records, "schemas": schemas, "checks": checks,
            "note": "Recorded inputs checked for internal consistency; external facts were not verified."});
        if let Some(waived) = waived {
            snapshot["waived"] = waived.clone();
        }
        snapshot
    }

    pub fn transition_workflow(
        &self,
        node: &Node,
        state: &str,
        evidence: &str,
    ) -> Result<Option<Value>> {
        // Every new transition, including re-entry into complete, is an action
        // under the current policy, regardless of the source node's lifecycle.
        let issues = self.current_workflow_issues(node, Some(state));
        if !issues.is_empty() {
            return Err(Error::input(issues.join("\n")));
        }
        if self.config.workflow.records.is_empty() && self.config.workflow.guards.is_empty() {
            return Ok(None);
        }
        let names: Vec<_> = self
            .config
            .workflow
            .records
            .iter()
            .filter(|(name, schema)| {
                schema.kinds.iter().any(|k| k == node.kind()) && node.attrs.contains_key(*name)
            })
            .map(|(name, _)| name.clone())
            .collect();
        let mut checks = vec![];
        let mut waived = Map::new();
        for (guard_name, guard) in &self.config.workflow.guards {
            if !guard.states.iter().any(|s| s == state) {
                continue;
            }
            match self.guard_waiver(node, guard) {
                Some(record) => {
                    waived.insert(guard_name.clone(), Value::String(record));
                }
                None => checks.extend(guard.checks.iter().cloned()),
            }
        }
        let waived = Value::Object(waived);
        Ok(Some(self.workflow_snapshot(
            node,
            &names,
            &checks,
            evidence,
            Some(&waived),
        )))
    }

    pub fn submit_record(&mut self, id: &str, name: &str, evidence: &str) -> Result<Value> {
        if evidence.trim().is_empty() {
            return Err(Error::input("--evidence is required to submit a record"));
        }
        let node = self.node(id)?;
        if node.attrs.contains_key("compressed") {
            return Err(Error::input(
                "Cannot submit records to a compressed requirement",
            ));
        }
        let mut issues = history_issues(node);
        issues.extend(self.record_issues(node, name));
        if !issues.is_empty() {
            return Err(Error::input(issues.join("\n")));
        }
        let snapshot = self.workflow_snapshot(node, &[name.to_string()], &[], evidence, None);
        let node = self.nodes.get_mut(id).unwrap();
        let history = node
            .attrs
            .entry("record_history")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .ok_or_else(|| Error::corrupt("record_history must be an array"))?;
        history.push(snapshot.clone());
        Ok(snapshot)
    }
}
