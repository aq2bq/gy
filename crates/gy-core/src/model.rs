use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{fmt, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct Error {
    pub code: u8,
    pub message: String,
}
impl Error {
    pub fn input(s: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: s.into(),
        }
    }
    pub fn corrupt(s: impl Into<String>) -> Self {
        Self {
            code: 3,
            message: s.into(),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::corrupt(e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;

pub const KINDS: &[(&str, &str, &str)] = &[
    ("need", "N-", "needs"),
    ("question", "Q-", "questions"),
    ("decision", "D-", "decisions"),
    ("requirement", "#", "requirements"),
    ("criterion", "AC-", "criteria"),
    ("gate", "G-", "gates"),
];
pub const STATES: &[&str] = &[
    "unfiled",
    "defining",
    "awaiting-design",
    "awaiting-approval",
    "awaiting-implementation",
    "awaiting-audit",
    "awaiting-pr",
    "awaiting-merge",
    "awaiting-production",
    "awaiting-cleanup",
    "complete",
];
pub const INTEGRITY_NOTE: &str = "This checks the graph's internal consistency. It does not verify that the ledger matches reality.";
pub fn base_state(s: &str) -> Option<&str> {
    STATES.iter().copied().find(|base| {
        s == *base
            || s.strip_prefix(base)
                .is_some_and(|tail| tail.starts_with(" (") && tail.ends_with(')') && tail.len() > 3)
    })
}
pub fn safe_component(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.chars().all(|c| c.is_alphanumeric() || "_- .".contains(c))
        && !s.contains('/')
        && !s.contains('\\')
}
pub fn valid_id(id: &str, kind: &str) -> bool {
    KINDS.iter().find(|x| x.0 == kind).is_some_and(|(_, p, _)| {
        id.strip_prefix(p).is_some_and(|n| {
            !n.is_empty()
                && n.bytes().all(|c| c.is_ascii_digit())
                && n.parse::<u64>().is_ok_and(|n| n > 0)
        })
    })
}
pub fn text(v: &Value) -> String {
    v.as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| v.to_string())
}
pub fn ids(v: Option<&Value>) -> Vec<String> {
    match v {
        None | Some(Value::Null) => vec![],
        Some(Value::Array(a)) => a.iter().flat_map(|v| ids(Some(v))).collect(),
        Some(Value::Object(m)) => m.get("id").map(text).into_iter().collect(),
        Some(v) => vec![text(v)],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub attrs: Map<String, Value>,
    pub body: String,
    #[serde(skip)]
    pub path: PathBuf,
}
impl Node {
    pub fn get(&self, key: &str) -> &str {
        self.attrs.get(key).and_then(Value::as_str).unwrap_or("")
    }
    pub fn id(&self) -> &str {
        self.get("id")
    }
    pub fn kind(&self) -> &str {
        self.get("type")
    }
    pub fn scope(&self) -> &str {
        self.get("scope")
    }
    pub fn refs(&self, key: &str) -> Vec<String> {
        ids(self.attrs.get(key))
    }
    pub fn closed(&self) -> bool {
        self.kind() == "question" && self.get("status") == "closed"
    }
    pub fn put(&mut self, key: &str, value: impl Serialize) {
        self.attrs.insert(
            key.into(),
            serde_json::to_value(value).expect("serializable value"),
        );
    }
    pub fn parse(raw: &str, path: PathBuf) -> Result<Self> {
        let raw = raw
            .strip_prefix('\u{feff}')
            .unwrap_or(raw)
            .replace("\r\n", "\n");
        let rest = raw.strip_prefix("---\n").ok_or_else(|| {
            Error::corrupt(format!("{}: missing YAML frontmatter", path.display()))
        })?;
        let (yaml, body) = rest.split_once("\n---\n").ok_or_else(|| {
            Error::corrupt(format!(
                "{}: missing closing frontmatter delimiter",
                path.display()
            ))
        })?;
        let attrs: Map<String, Value> = serde_yaml::from_str(yaml)
            .map_err(|e| Error::corrupt(format!("{}: {e}", path.display())))?;
        let n = Self {
            attrs,
            body: body.into(),
            path,
        };
        for key in ["id", "type", "title", "scope", "created"] {
            if n.get(key).trim().is_empty() {
                return Err(Error::corrupt(format!(
                    "{}: required attribute {key} is empty",
                    n.path.display()
                )));
            }
        }
        if !valid_id(n.id(), n.kind())
            || !safe_component(n.scope())
            || chrono::NaiveDate::parse_from_str(n.get("created"), "%Y-%m-%d").is_err()
        {
            return Err(Error::corrupt(format!(
                "{}: invalid id / type / scope / created format",
                n.path.display()
            )));
        }
        Ok(n)
    }
    pub fn markdown(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.attrs).map_err(|e| Error::corrupt(e.to_string()))?;
        Ok(format!("---\n{yaml}---\n{}", self.body))
    }
    pub fn new(id: &str, kind: &str, title: &str, scope: &str) -> Self {
        let attrs = json!({"id": id, "type": kind, "title":title, "scope":scope, "created":chrono::Utc::now().format("%Y-%m-%d").to_string()}).as_object().unwrap().clone();
        Self {
            attrs,
            body: if kind == "decision" {
                "\n## Context\n\n## Decision\n\n## Consequences\n".into()
            } else {
                "\n".into()
            },
            path: PathBuf::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub parent_issue: Option<u64>,
    pub scopes: std::collections::BTreeMap<String, ScopeConfig>,
    pub lint: std::collections::BTreeMap<String, RuleConfig>,
    pub render: RenderConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScopeConfig {
    pub parent_issue: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    Enabled(bool),
    Severity(String),
    Detail {
        enabled: Option<bool>,
        severity: Option<String>,
    },
}
impl RuleConfig {
    pub fn severity(&self) -> Result<Option<&str>> {
        let s = match self {
            Self::Enabled(false) => return Ok(None),
            Self::Enabled(true) => "error",
            Self::Severity(s) => s,
            Self::Detail {
                enabled: Some(false),
                ..
            } => return Ok(None),
            Self::Detail { severity, .. } => severity.as_deref().unwrap_or("error"),
        };
        if s == "off" {
            Ok(None)
        } else if ["error", "warn"].contains(&s) {
            Ok(Some(s))
        } else {
            Err(Error::corrupt(format!(
                "Invalid lint severity {s}; use error / warn / off"
            )))
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    pub split_threshold: usize,
    pub output: String,
}
impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            split_threshold: 100,
            output: "{scope}/README.md".into(),
        }
    }
}

pub fn today() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Edge endpoints may allow more than one node type.
pub fn kind_matches(actual: &str, allowed: &str) -> bool {
    allowed.split('|').any(|kind| actual == kind)
}
