//! The reading commands that need more than a single view call: list and the
//! publish reading.
use crate::output::emit;
use crate::repo;
use crate::{Cli, Command};
use gy_ledger::{Error, Filter, NodeKind, Result, config, describe, list, publish};
use std::path::{Path, PathBuf};

pub fn read_list(cli: &Cli, ledger: &Path) -> Result<()> {
    let Command::List {
        kind,
        status,
        targets,
        grep,
        actor,
        since,
    } = &cli.command
    else {
        return Err(Error::invalid("not a list"));
    };
    let repository = repo::open(ledger)?;
    let filter = Filter {
        kind: kind.as_deref().map(parse_kind).transpose()?,
        status: status.clone(),
        targets: targets
            .as_deref()
            .map(|text| repository.resolve(text))
            .transpose()?,
        grep: grep.clone(),
        actor: actor.clone(),
        since: *since,
    };
    emit(cli.json, &list(&repository, &filter)?)
}

/// The reading goes to `--out`, else gy.toml's `output`, else stdout. Naming
/// nodes switches to their description only (D-87).
pub fn write_publish(
    cli: &Cli,
    root: &Path,
    ledger: &Path,
    ids: &[String],
    since: Option<u64>,
    out: Option<&Path>,
) -> Result<()> {
    let repository = repo::open(ledger)?;
    let text = if ids.is_empty() {
        publish(&repository, cli.scope.as_deref(), since)?
    } else {
        let resolved = ids
            .iter()
            .map(|text| repository.resolve(text))
            .collect::<Result<Vec<_>>>()?;
        describe(&repository, &resolved)?
    };
    let path = match out {
        Some(path) => Some(path.to_path_buf()),
        None => output_path(root)?,
    };
    match path {
        Some(path) => write_file(&path, &text),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

fn output_path(root: &Path) -> Result<Option<PathBuf>> {
    Ok(config::read(root)?.output.map(|out| root.join(out)))
}

fn write_file(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

fn parse_kind(text: &str) -> Result<NodeKind> {
    NodeKind::ALL
        .into_iter()
        .find(|kind| kind.name() == text || kind.prefix() == text)
        .ok_or_else(|| Error::invalid(format!("unknown type {text}")))
}
