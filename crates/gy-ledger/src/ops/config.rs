//! gy.toml: only `[scopes.<name>]` and a top-level `output` are allowed; any
//! other key is an error named on load (AC-40).
use crate::store::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
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
    /// The git URL of the ledger's remote, when the ledger is a team copy
    /// (n-6f47, d-39f6). Absent leaves the ledger local, exactly as before.
    #[serde(default)]
    pub remote: Option<String>,
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

/// Rename a scope table in gy.toml, leaving the other bytes (comments, order,
/// and the other keys) as they were (n-24dd). A missing old table or an
/// existing new one is an error, so the caller can roll the log back.
pub fn rename_scope(root: &Path, from: &str, to: &str) -> Result<()> {
    let path = root.join("gy.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|_| Error::invalid(format!("no gy.toml at {}", path.display())))?;
    let old = format!("[scopes.{from}]");
    let new = format!("[scopes.{to}]");
    if !text.contains(&old) {
        return Err(Error::invalid(format!("gy.toml has no [scopes.{from}]")));
    }
    if text.contains(&new) {
        return Err(Error::invalid(format!("gy.toml already has [scopes.{to}]")));
    }
    std::fs::write(&path, text.replace(&old, &new))
        .map_err(|error| Error::invalid(format!("gy.toml: {error}")))
}

/// Write the ledger's `remote` into gy.toml, just before the first `[scopes.*]`
/// so it stays a top-level key, or at the end when there is no scope table
/// (n-57c5, ac-efd7). The other bytes (comments, order, the other keys) are
/// left as they were.
pub fn write_remote(root: &Path, url: &str) -> Result<()> {
    let path = root.join("gy.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|_| Error::invalid(format!("no gy.toml at {}", path.display())))?;
    let line = format!("remote = \"{url}\"\n");
    let out = match text.find("[scopes.") {
        Some(at) => format!("{}{}{}", &text[..at], line, &text[at..]),
        None => {
            let mut out = text;
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&line);
            out
        }
    };
    std::fs::write(&path, out).map_err(|error| Error::invalid(format!("gy.toml: {error}")))
}

/// A scope name must be a TOML bare key: `A-Za-z0-9_-` only, at least one
/// character (n-29b3). Anything else would need a quoted key, which the plain
/// `[scopes.<name>]` matching in `rename_scope` and the scope word in
/// `--scope` and publish would all have to learn.
pub fn check_scope_name(name: &str) -> Result<()> {
    let bare = !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    if bare {
        Ok(())
    } else {
        Err(Error::invalid(format!(
            "scope names take A-Za-z0-9_- only, got {name}"
        )))
    }
}

/// Start a repository: write a gy.toml holding one scope and nothing else. The
/// ledger is not touched; the first write makes it (n-29b3, N-63). The caller
/// resolves where `root` is and never calls this over an existing file.
pub fn init(root: &Path, name: &str) -> Result<()> {
    check_scope_name(name)?;
    let path = root.join("gy.toml");
    if path.is_file() {
        return Err(Error::invalid(format!(
            "gy.toml already exists at {}",
            path.display()
        )));
    }
    std::fs::write(&path, format!("[scopes.{name}]\n"))
        .map_err(|error| Error::invalid(format!("gy.toml: {error}")))
}

/// The tail every `gy init` prints: the skill to read, then the first node,
/// with no optional flags so a first-time agent has no branch to take
/// (n-29b3, contract 4). The two commands keep their indent, so a reader sees
/// at a glance what to type; a `\` continuation would eat it.
const NEXT: &str = concat!(
    "Next: read the gy-loop skill, the entry point your agent loads for this project.\n",
    "Then file your first node:\n",
    "  export GY_ACTOR=<your name>\n",
    "  gy criterion add \"<what must hold>\"",
);

/// What `gy init` reports: where the repository is, its scopes, its remote,
/// whether this run wrote gy.toml, and the lines that lead a first-time agent
/// from here to its first node (n-29b3, contract 4).
#[derive(Debug, Serialize)]
pub struct Init {
    pub root: String,
    pub scopes: Vec<String>,
    pub remote: Option<String>,
    pub created: bool,
    pub next: Vec<String>,
}

impl Init {
    /// A run that wrote gy.toml here, with the one scope it named.
    pub fn created(root: &Path, name: &str) -> Self {
        Self {
            root: root.display().to_string(),
            scopes: vec![name.to_string()],
            remote: None,
            created: true,
            next: next_lines(),
        }
    }

    /// A run that found gy.toml already: what it holds, untouched.
    pub fn existing(root: &Path, config: &Config) -> Self {
        Self {
            root: root.display().to_string(),
            scopes: config.scopes.keys().cloned().collect(),
            remote: config.remote.clone(),
            created: false,
            next: next_lines(),
        }
    }
}

/// The next-step lines, the same for both reports.
fn next_lines() -> Vec<String> {
    NEXT.lines().map(str::to_string).collect()
}

impl fmt::Display for Init {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scopes = self.scopes.join(", ");
        if self.created {
            writeln!(f, "gy.toml written: {} (scope: {scopes})", self.root)?;
            writeln!(
                f,
                "The ledger appears with your first write; there is nothing else to set up."
            )?;
        } else {
            let remote = self.remote.as_deref().unwrap_or("local");
            writeln!(
                f,
                "gy.toml already here: {} (scope: {scopes}, remote: {remote})",
                self.root
            )?;
            writeln!(f, "Nothing was written.")?;
        }
        f.write_str("\n")?;
        for line in &self.next {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}
