//! Record schemas and their validation.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
#[non_exhaustive]
pub struct WorkflowConfig {
    pub records: BTreeMap<String, RecordSchema>,
    pub guards: BTreeMap<String, StateGuard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RecordSchema {
    /// Project-authored explanation; never interpreted by validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
#[non_exhaustive]
pub struct FieldSchema {
    /// Project-authored explanation; never interpreted by validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
#[non_exhaustive]
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
#[non_exhaustive]
pub struct StateGuard {
    pub states: Vec<String>,
    pub records: Vec<String>,
    pub checks: Vec<RecordCheck>,
    /// Name of a project-defined record that waives this guard when valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waived_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
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
#[non_exhaustive]
pub enum CheckKind {
    Equal,
    SameSet,
    Subset,
    MatchesDeclaredFiles,
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
            if let Some(name) = &guard.waived_by {
                if !self
                    .records
                    .get(name)
                    .is_some_and(|s| s.kinds.iter().any(|k| k == "requirement"))
                {
                    return Err(bad(format!(
                        "{state} waived_by names unknown or non-requirement record {name}"
                    )));
                }
            }
            for check in &guard.checks {
                if check.left_keys.len() != check.right_keys.len()
                    || (!check.left_keys.is_empty()
                        && matches!(
                            check.kind,
                            CheckKind::Equal | CheckKind::MatchesDeclaredFiles
                        ))
                    || check
                        .left_keys
                        .iter()
                        .chain(&check.right_keys)
                        .any(|k| !component(k))
                {
                    return Err(bad(format!("{state}: invalid comparison keys")));
                }
                if matches!(check.kind, CheckKind::MatchesDeclaredFiles) {
                    for (path, item_kind) in [
                        (&check.left, FieldType::String),
                        (&check.right, FieldType::Object),
                    ] {
                        let mut parts = path.split('.');
                        let field =
                            self.records
                                .get(parts.next().unwrap_or_default())
                                .and_then(|record| {
                                    schema_field(&record.fields, &parts.collect::<Vec<_>>())
                                });
                        if path.contains("[]")
                            || !field.is_some_and(|f| {
                                f.kind == FieldType::Array
                                    && f.items.as_deref().is_some_and(|i| i.kind == item_kind)
                            })
                        {
                            return Err(bad(format!(
                                "{state}: matches-declared-files requires an array of {item_kind:?} at {path}, without [] projections"
                            )));
                        }
                    }
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
    parts.is_empty() || schema_field(fields, parts).is_some()
}
fn schema_field<'a>(
    fields: &'a BTreeMap<String, FieldSchema>,
    parts: &[&str],
) -> Option<&'a FieldSchema> {
    let part = *parts.first()?;
    let mut field = fields.get(part.strip_suffix("[]").unwrap_or(part))?;
    if part.ends_with("[]") {
        if field.kind != FieldType::Array {
            return None;
        }
        field = field.items.as_deref()?;
    }
    if parts.len() == 1 {
        return Some(field);
    }
    if field.kind != FieldType::Object {
        return None;
    }
    schema_field(&field.fields, &parts[1..])
}
pub(super) fn validate_object(
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
