//! Freezing a requirement's 0.4 records as a publication (N-69). The file is
//! the original markdown; the node keeps only the relative path.
use crate::legacy::Legacy;
use gy_ledger::{Node, Result};
use std::path::Path;

/// Write each requirement that carries records to `<dir>/<scope>/<old id>.md`
/// and leave the relative path in the node's `legacy_records` free attribute.
/// Returns how many were written.
pub fn freeze(legacy: &Legacy, nodes: &mut [Node], dir: &Path) -> Result<usize> {
    let mut count = 0;
    for (source, node) in legacy.nodes.iter().zip(nodes.iter_mut()) {
        if !source.has_records() {
            continue;
        }
        let relative = format!("{}/{}.md", source.scope, source.id);
        write(&dir.join(&relative), &source.raw)?;
        node.set_free("legacy_records", relative);
        count += 1;
    }
    Ok(count)
}

fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
}
