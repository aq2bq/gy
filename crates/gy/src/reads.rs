//! The reading commands that need more than a single view call: list and the
//! publish reading.
use crate::output::{emit, write};
use crate::repo;
use crate::{Cli, Command};
use gy_ledger::{Actor, Error, Filter, NodeKind, Result, config, list, log_seq, publish};
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

/// The publication goes to `--out`, else gy.toml's `output`, else stdout. A
/// `{seq}` in the path becomes the write sequence.
pub fn write_publish(
    cli: &Cli,
    root: &Path,
    ledger: &Path,
    since: Option<u64>,
    out: Option<&Path>,
) -> Result<()> {
    let repository = repo::open(ledger)?;
    let writer = Actor::from_env()
        .map(|actor| actor.name().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let text = publish(
        &repository,
        cli.scope.as_deref(),
        since,
        &writer,
        &ledger.display().to_string(),
    )?;
    let path = match out {
        Some(path) => Some(path.to_path_buf()),
        None => output_path(root)?,
    };
    match path {
        Some(path) => write_file(&substitute(&path, log_seq(&repository)), &text),
        None => write(&text),
    }
}

fn substitute(path: &Path, seq: u64) -> PathBuf {
    PathBuf::from(path.to_string_lossy().replace("{seq}", &seq.to_string()))
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
