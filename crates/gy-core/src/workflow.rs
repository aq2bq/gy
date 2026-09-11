//! Configured record checks inspect reports, never external systems.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorkflowConfig {
    pub records: BTreeMap<String, RecordSchema>,
    pub guards: BTreeMap<String, StateGuard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordSchema {
    pub kinds: Vec<String>,
    #[serde(default)]
    pub required: bool,
    /// A top-level string field identifying an immutable recorded version.
    pub version_field: Option<String>,
    pub fields: BTreeMap<String, FieldSchema>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSchema {
    #[serde(rename = "type")]
    pub kind: FieldType,
    #[serde(default = "yes")]
    pub required: bool,
    #[serde(default)]
    pub allow_empty: bool,
    #[serde(default)]
    pub values: Vec<String>,
    pub equals: Option<Value>,
    #[serde(default)]
    pub fields: BTreeMap<String, FieldSchema>,
    pub items: Option<Box<FieldSchema>>,
    /// Discriminator for explicit alternatives, such as normal/design-waived.
    pub variant_field: Option<String>,
    #[serde(default)]
    pub variants: BTreeMap<String, BTreeMap<String, FieldSchema>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldType {
    String,
    Url,
    Boolean,
    Integer,
    Number,
    Array,
    Object,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StateGuard {
    pub states: Vec<String>,
    pub records: Vec<String>,
    pub checks: Vec<RecordCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordCheck {
    pub kind: CheckKind,
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub left_keys: Vec<String>,
    #[serde(default)]
    pub right_keys: Vec<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckKind {
    Equal,
    SameSet,
    Subset,
}

fn component(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

impl WorkflowConfig {
    pub fn validate(&self) -> Result<()> {
        let bad = |s: String| Error::corrupt(format!("workflow: {s}"));
        for (name, schema) in &self.records {
            if !component(name)
                || [
                    "id",
                    "type",
                    "title",
                    "scope",
                    "created",
                    "status",
                    "transitions",
                    "record_history",
                    "compressed",
                    "compressed_from",
                    "evidence",
                    "capture",
                    "closed_at",
                    "closed_by",
                    "satisfied",
                    "satisfied_at",
                    "decision_scope",
                ]
                .contains(&name.as_str())
                || EDGES.iter().any(|e| name == e.0 || name == e.3)
                || REFERENCE_KEYS.contains(&name.as_str())
            {
                return Err(bad(format!("invalid or reserved record name {name}")));
            }
            if schema.kinds.is_empty()
                || schema.kinds.iter().any(|k| !KINDS.iter().any(|x| x.0 == k))
            {
                return Err(bad(format!("{name}.kinds must name existing node types")));
            }
            if schema.fields.is_empty() {
                return Err(bad(format!("{name}.fields is empty")));
            }
            validate_fields(&schema.fields, name)?;
            if let Some(version) = &schema.version_field {
                if !schema
                    .fields
                    .get(version)
                    .is_some_and(|f| f.kind == FieldType::String && f.required && !f.allow_empty)
                {
                    return Err(bad(format!(
                        "{name}.version_field must name a required nonempty string field"
                    )));
                }
            }
        }
        for (state, guard) in &self.guards {
            if !component(state)
                || guard.states.is_empty()
                || guard.records.is_empty()
                || guard.states.iter().any(|s| !STATES.contains(&s.as_str()))
            {
                return Err(bad(format!("unknown requirement state {state}")));
            }
            for name in &guard.records {
                if !self
                    .records
                    .get(name)
                    .is_some_and(|s| s.kinds.iter().any(|k| k == "requirement"))
                {
                    return Err(bad(format!(
                        "{state} requires unknown or non-requirement record {name}"
                    )));
                }
            }
            for check in &guard.checks {
                if check.left_keys.len() != check.right_keys.len()
                    || (!check.left_keys.is_empty() && matches!(check.kind, CheckKind::Equal))
                    || check
                        .left_keys
                        .iter()
                        .chain(&check.right_keys)
                        .any(|k| !component(k))
                {
                    return Err(bad(format!("{state}: invalid comparison keys")));
                }
                for (path, keys) in [
                    (&check.left, &check.left_keys),
                    (&check.right, &check.right_keys),
                ] {
                    if !valid_record_path(path)
                        || !guard
                            .records
                            .iter()
                            .any(|r| path.split('.').next() == Some(r))
                    {
                        return Err(bad(format!(
                            "{state}: check path {path} must start with a required record"
                        )));
                    }
                    // Validate paths against every branch they traverse, so a typo
                    // cannot turn a configured comparison into an empty comparison.
                    let mut parts = path.split('.');
                    let schema = &self.records[parts.next().unwrap()];
                    let parts: Vec<_> = parts.collect();
                    if !schema_path(&schema.fields, &parts) {
                        return Err(bad(format!("{state}: unknown check path {path}")));
                    }
                    for key in keys {
                        let mut projected = parts.clone();
                        let Some(last) = projected.pop() else {
                            return Err(bad(format!(
                                "{state}: keyed comparisons require an array field"
                            )));
                        };
                        let expanded = if last.ends_with("[]") {
                            last.to_string()
                        } else {
                            format!("{last}[]")
                        };
                        projected.push(&expanded);
                        projected.push(key);
                        if !schema_path(&schema.fields, &projected) {
                            return Err(bad(format!(
                                "{state}: unknown comparison key {path}.{key}"
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
fn validate_fields(fields: &BTreeMap<String, FieldSchema>, path: &str) -> Result<()> {
    for (name, field) in fields {
        if !component(name) {
            return Err(Error::corrupt(format!(
                "workflow: invalid field {path}.{name}"
            )));
        }
        validate_field(field, &format!("{path}.{name}"))?;
    }
    Ok(())
}
fn validate_field(f: &FieldSchema, path: &str) -> Result<()> {
    let error = || Error::corrupt(format!("workflow: inconsistent field schema {path}"));
    if !f.values.is_empty() && !matches!(f.kind, FieldType::String | FieldType::Url) {
        return Err(error());
    }
    if f.kind == FieldType::Array {
        let Some(items) = &f.items else {
            return Err(error());
        };
        validate_field(items, path)?;
    } else if f.items.is_some() {
        return Err(error());
    }
    if f.kind != FieldType::Object
        && (!f.fields.is_empty() || f.variant_field.is_some() || !f.variants.is_empty())
    {
        return Err(error());
    }
    validate_fields(&f.fields, path)?;
    match &f.variant_field {
        Some(tag) => {
            if f.variants.is_empty()
                || !f
                    .fields
                    .get(tag)
                    .is_some_and(|s| s.kind == FieldType::String && s.required && !s.allow_empty)
            {
                return Err(error());
            }
            for (variant, fields) in &f.variants {
                if variant.trim().is_empty() || fields.keys().any(|k| f.fields.contains_key(k)) {
                    return Err(error());
                }
                validate_fields(fields, &format!("{path}.{variant}"))?;
            }
        }
        None if !f.variants.is_empty() => return Err(error()),
        None => (),
    }
    if let Some(expected) = &f.equals {
        let mut issues = vec![];
        validate_value(Some(expected), f, path, &mut issues);
        if !issues.is_empty() {
            return Err(error());
        }
    }
    Ok(())
}
fn valid_record_path(path: &str) -> bool {
    path.split('.')
        .all(|part| component(part.strip_suffix("[]").unwrap_or(part)))
        && !path.split('.').next().unwrap_or("").ends_with("[]")
}
fn schema_path(fields: &BTreeMap<String, FieldSchema>, parts: &[&str]) -> bool {
    if parts.is_empty() {
        return true;
    }
    let part = parts[0];
    let Some(mut field) = fields.get(part.strip_suffix("[]").unwrap_or(part)) else {
        return false;
    };
    if part.ends_with("[]") {
        if field.kind != FieldType::Array {
            return false;
        }
        let Some(item) = field.items.as_deref() else {
            return false;
        };
        field = item;
    }
    if parts.len() == 1 {
        return true;
    }
    field.kind == FieldType::Object && schema_path(&field.fields, &parts[1..])
}
fn validate_object(
    value: &Value,
    fields: &BTreeMap<String, FieldSchema>,
    path: &str,
    issues: &mut Vec<String>,
) {
    let Some(object) = value.as_object() else {
        issues.push(format!("{path} must be an object"));
        return;
    };
    for (key, field) in fields {
        validate_value(object.get(key), field, &format!("{path}.{key}"), issues);
    }
}
fn validate_value(value: Option<&Value>, f: &FieldSchema, path: &str, issues: &mut Vec<String>) {
    let Some(value) = value else {
        if f.required {
            issues.push(format!("{path} is required"));
        }
        return;
    };
    let valid = match f.kind {
        FieldType::String => value.is_string(),
        FieldType::Url => value.as_str().is_some_and(valid_url),
        FieldType::Boolean => value.is_boolean(),
        FieldType::Integer => value.is_i64() || value.is_u64(),
        FieldType::Number => value.is_number(),
        FieldType::Array => value.is_array(),
        FieldType::Object => value.is_object(),
    };
    if !valid {
        issues.push(format!("{path} must have type {:?}", f.kind));
        return;
    }
    if !f.allow_empty
        && (value.as_str().is_some_and(|s| s.trim().is_empty())
            || value.as_array().is_some_and(Vec::is_empty)
            || value.as_object().is_some_and(Map::is_empty))
    {
        issues.push(format!("{path} must not be empty"));
    }
    if f.equals.as_ref().is_some_and(|expected| value != expected) {
        issues.push(format!("{path} must equal {}", f.equals.as_ref().unwrap()));
    }
    if !f.values.is_empty()
        && !value
            .as_str()
            .is_some_and(|s| f.values.iter().any(|v| v == s))
    {
        issues.push(format!("{path} must be one of {}", f.values.join(", ")));
    }
    if let Some(items) = &f.items {
        for (index, item) in value.as_array().unwrap().iter().enumerate() {
            validate_value(Some(item), items, &format!("{path}[{index}]"), issues);
        }
    }
    if f.kind == FieldType::Object {
        validate_object(value, &f.fields, path, issues);
        if let Some(tag) = &f.variant_field {
            match value
                .get(tag)
                .and_then(Value::as_str)
                .and_then(|v| f.variants.get(v))
            {
                Some(fields) => validate_object(value, fields, path, issues),
                None => issues.push(format!(
                    "{path}.{tag} must select one of {}",
                    f.variants.keys().cloned().collect::<Vec<_>>().join(", ")
                )),
            }
        }
    }
}
fn path_values<'a>(node: &'a Node, path: &str) -> Option<Vec<&'a Value>> {
    let mut parts = path.split('.');
    let root = node.attrs.get(parts.next()?)?;
    let mut values = vec![root];
    for part in parts {
        let expand = part.ends_with("[]");
        let key = part.strip_suffix("[]").unwrap_or(part);
        let mut next = vec![];
        for value in values {
            let value = value.get(key)?;
            if expand {
                next.extend(value.as_array()?.iter());
            } else {
                next.push(value);
            }
        }
        values = next;
    }
    Some(values)
}
fn string_set(values: Vec<&Value>, keys: &[String]) -> Option<BTreeSet<Vec<String>>> {
    let mut set = BTreeSet::new();
    for value in values {
        let items: Vec<_> = match value {
            Value::Array(a) => a.iter().collect(),
            v => vec![v],
        };
        for item in items {
            let values = if keys.is_empty() {
                vec![item]
            } else {
                keys.iter()
                    .map(|key| item.get(key))
                    .collect::<Option<Vec<_>>>()?
            };
            let tuple = values
                .into_iter()
                .map(|value| {
                    value
                        .as_str()
                        .filter(|s| !s.trim().is_empty())
                        .map(str::to_owned)
                })
                .collect::<Option<Vec<_>>>()?;
            set.insert(tuple);
        }
    }
    Some(set)
}
fn check_records(node: &Node, checks: &[RecordCheck]) -> Vec<String> {
    let mut issues = vec![];
    for check in checks {
        let (Some(left), Some(right)) = (
            path_values(node, &check.left),
            path_values(node, &check.right),
        ) else {
            issues.push(format!(
                "Cannot compare {} and {}: a path is missing or has the wrong type",
                check.left, check.right
            ));
            continue;
        };
        let valid = match check.kind {
            CheckKind::Equal => {
                left.len() == 1 && right.len() == 1 && left == right && !left[0].is_null()
            }
            CheckKind::SameSet | CheckKind::Subset => match (
                string_set(left, &check.left_keys),
                string_set(right, &check.right_keys),
            ) {
                (Some(left), Some(right)) => match check.kind {
                    CheckKind::SameSet => left == right,
                    CheckKind::Subset => left.is_subset(&right),
                    _ => unreachable!(),
                },
                _ => false,
            },
        };
        if !valid {
            issues.push(format!(
                "{:?} check failed: {} versus {} (set operands must contain nonempty strings)",
                check.kind, check.left, check.right
            ));
        }
    }
    issues
}
fn snapshots(node: &Node) -> impl Iterator<Item = &Value> {
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

fn history_issues(node: &Node) -> Vec<String> {
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
        for guard in &guards {
            names.extend(guard.records.iter().cloned());
        }
        let mut issues: Vec<_> = history_issues
            .into_iter()
            .chain(names.iter().flat_map(|name| self.record_issues(node, name)))
            .collect();
        for guard in guards {
            issues.extend(check_records(node, &guard.checks));
        }
        issues
    }

    fn workflow_snapshot(
        &self,
        node: &Node,
        names: &[String],
        checks: &[RecordCheck],
        evidence: &str,
    ) -> Value {
        let records: Map<_, _> = names
            .iter()
            .filter_map(|name| node.attrs.get(name).map(|v| (name.clone(), v.clone())))
            .collect();
        let schemas: BTreeMap<_, _> = names
            .iter()
            .map(|name| (name, &self.config.workflow.records[name]))
            .collect();
        json!({"at": today(), "evidence": evidence, "records": records, "schemas": schemas, "checks": checks,
            "note": "Recorded inputs checked for internal consistency; external facts were not verified."})
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
        let checks: Vec<_> = self
            .config
            .workflow
            .guards
            .values()
            .filter(|g| g.states.iter().any(|s| s == state))
            .flat_map(|g| g.checks.iter().cloned())
            .collect();
        Ok(Some(
            self.workflow_snapshot(node, &names, &checks, evidence),
        ))
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
        let snapshot = self.workflow_snapshot(node, &[name.to_string()], &[], evidence);
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
