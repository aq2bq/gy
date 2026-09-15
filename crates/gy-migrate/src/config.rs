//! Generating and checking `<root>/gy.toml`, which holds only `[scopes.<name>]`
//! (AC-40). An existing file is never overwritten.
use crate::legacy::Legacy;
use gy_ledger::{Error, Result};
use std::collections::BTreeSet;
use std::path::Path;

/// The scopes the ledger uses.
pub fn scopes(legacy: &Legacy) -> BTreeSet<String> {
    legacy.nodes.iter().map(|node| node.scope.clone()).collect()
}

/// Write `[scopes.<name>]` for every scope used, or, when gy.toml exists, check
/// that it names them all and fail if any is missing.
pub fn ensure(root: &Path, legacy: &Legacy) -> Result<()> {
    let scopes = scopes(legacy);
    let path = root.join("gy.toml");
    if path.is_file() {
        let text = std::fs::read_to_string(&path)?;
        let missing: Vec<String> = scopes
            .iter()
            .filter(|scope| !text.contains(&format!("[scopes.{scope}]")))
            .cloned()
            .collect();
        if !missing.is_empty() {
            return Err(Error::invalid(format!(
                "{} does not name scopes: {}",
                path.display(),
                missing.join(", ")
            )));
        }
        return Ok(());
    }
    let body: String = scopes
        .iter()
        .map(|scope| format!("[scopes.{scope}]\n"))
        .collect();
    std::fs::write(&path, body)?;
    Ok(())
}
