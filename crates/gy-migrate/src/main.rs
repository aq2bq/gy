//! gy-migrate: read a 0.4 ledger and write it into the new canonical ledger
//! (N-67). Edges and frozen records arrive with the following step.
mod legacy;
mod nodes;

use clap::Parser;
use gy_ledger::{
    Actor, Error, FileStore, FormatVersion, NodeId, Repository, Result, format, location,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gy-migrate",
    version,
    about = "Migrate a 0.4 ledger into the new gy"
)]
struct Cli {
    /// The 0.4 ledger directory (holds gy.toml and the scope directories).
    ledger: PathBuf,
    /// The repository root that will hold the new gy.toml.
    #[arg(long, value_name = "DIR")]
    root: PathBuf,
    /// Prefix for a requirement reference made from its old #N id.
    #[arg(long = "ref-base", value_name = "URL")]
    ref_base: Option<String>,
    /// Where frozen records are written (used by the next step).
    #[arg(long, value_name = "DIR")]
    publication: Option<PathBuf>,
    /// Read and report only; write nothing.
    #[arg(long)]
    dry_run: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    guard(&cli)?;
    let legacy = legacy::read(&cli.ledger)?;
    let written = if cli.dry_run {
        0
    } else {
        write(&cli, &legacy)?
    };
    print!("{}", legacy.report());
    if !cli.dry_run {
        println!("written: {written} nodes");
    }
    settings(&cli);
    Ok(())
}

/// The canonical ledger must not exist yet, and a real run names its actor.
fn guard(cli: &Cli) -> Result<()> {
    let ledger = location::ledger_dir(&cli.root);
    if ledger.join(format::FILE).is_file() {
        return Err(Error::invalid(format!(
            "the canonical ledger already exists at {}",
            ledger.display()
        )));
    }
    if !cli.dry_run {
        Actor::from_env()?;
    }
    Ok(())
}

/// Build every new node, then write them in one transaction.
fn write(cli: &Cli, legacy: &legacy::Legacy) -> Result<usize> {
    let ledger = location::ledger_dir(&cli.root);
    format::write(&ledger, FormatVersion::CURRENT)?;
    let mut repository = Repository::new(FileStore::open(&ledger)?);
    let derived = legacy.derived_ids();
    let mut nodes = Vec::new();
    for node in &legacy.nodes {
        let id = NodeId::mint(nodes::kind(node.new_kind())?, repository.store_mut())?;
        let derive = derived.contains(&node.id);
        nodes.push(nodes::build(node, id, cli.ref_base.as_deref(), derive)?);
    }
    let source = cli.ledger.display().to_string();
    repository.transaction("migrate from 0.4", &source, |repository| {
        for node in &nodes {
            repository.put(node)?;
        }
        Ok(())
    })?;
    Ok(nodes.len())
}

fn settings(cli: &Cli) {
    if let Some(base) = &cli.ref_base {
        println!("ref base: {base}");
    }
    if let Some(dir) = &cli.publication {
        println!("publication: {}", dir.display());
    }
}
