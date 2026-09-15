//! gy.toml: only `[scopes.<name>]` and a top-level `output` are allowed; any
//! other key is an error named on load (AC-40).
use crate::store::{Error, Result};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

/// The configuration. `deny_unknown_fields` rejects the old keys
/// (`parent_issue`, `lint`, `render`, `import`, `workflow`, and so on).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub scopes: BTreeMap<String, Scope>,
    #[serde(default)]
    pub output: Option<String>,
}

/// A scope's settings. Empty for now; a table so later items have a home.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {}

pub fn read(root: &Path) -> Result<Config> {
    let path = root.join("gy.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|_| Error::invalid(format!("no gy.toml at {}", path.display())))?;
    toml::from_str(&text).map_err(|error| Error::invalid(format!("gy.toml: {error}")))
}
