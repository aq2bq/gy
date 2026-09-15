//! The reading commands that need more than a single view call: list and the
//! publish reading.
use crate::output::emit;
use crate::repo;
use crate::{Cli, Command};
use gy_ledger::{
    Actor, Error, Filter, NodeKind, Publication, Repository, Result, config, file, list, publish,
};
use gy_serve::server::Opened;
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

/// Serve the ledger over HTTP on localhost until stopped (n-a493). The CLI
/// only wires it: gy-serve owns the server, and no write path exists. A read
/// may name a sequence, which opens the ledger as it stood then (n-10e1).
pub fn serve(ledger: &Path) -> Result<()> {
    let ledger = ledger.to_path_buf();
    gy_serve::server::serve(Box::new(move |at| match at {
        None => repo::open(&ledger).map(Opened::Now),
        Some(seq) => {
            // The same reader actor repo::open uses; open_at never writes.
            let reader = |_: &str| Some("gy-read".to_string());
            file::open_at(&ledger, seq, reader).map(|store| Opened::At(Repository::new(store)))
        }
    }))
}

/// The publication goes under `--out`, else gy.toml's `output` (an error when
/// neither is given). Each target scope's directory is removed and rewritten.
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
        .unwrap_or_default();
    let publication = publish(
        &repository,
        cli.scope.as_deref(),
        since,
        &writer,
        &ledger.display().to_string(),
    )?;
    let out = match out {
        Some(path) => path.to_path_buf(),
        None => output_path(root)?
            .ok_or_else(|| Error::invalid("publish needs --out or gy.toml output"))?,
    };
    write_publication(&out, &publication)
}

fn write_publication(out: &Path, publication: &Publication) -> Result<()> {
    for scope in &publication.scopes {
        let dir = out.join(&scope.name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        std::fs::create_dir_all(&dir)?;
        for file in &scope.files {
            let path = dir.join(&file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, &file.text)?;
        }
    }
    Ok(())
}

fn output_path(root: &Path) -> Result<Option<PathBuf>> {
    Ok(config::read(root)?.output.map(|out| root.join(out)))
}

fn parse_kind(text: &str) -> Result<NodeKind> {
    NodeKind::ALL
        .into_iter()
        .find(|kind| kind.name() == text || kind.prefix() == text)
        .ok_or_else(|| Error::invalid(format!("unknown type {text}")))
}
