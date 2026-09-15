//! gy-migrate: read a 0.4 ledger and write it into the new canonical ledger
//! (N-67), with its edges (N-66). Frozen records arrive with the next step.
mod edges;
mod legacy;
mod nodes;
mod report;

use clap::Parser;
use gy_ledger::{
    Actor, Error, FileStore, FormatVersion, NodeId, Repository, Result, format, location,
};
use std::collections::BTreeMap;
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
    print!("{}", legacy.report());
    if cli.dry_run {
        settings(&cli);
        return Ok(());
    }
    let (written, diagnostics) = write(&cli, &legacy)?;
    println!("written: {written} nodes");
    print!("{diagnostics}");
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

/// Build every new node and its edges, then write them in one transaction.
fn write(cli: &Cli, legacy: &legacy::Legacy) -> Result<(usize, edges::Edges)> {
    let ledger = location::ledger_dir(&cli.root);
    format::write(&ledger, FormatVersion::CURRENT)?;
    let mut repository = Repository::new(FileStore::open(&ledger)?);
    let derived = legacy.derived_ids();
    let mut ids = BTreeMap::new();
    let mut nodes = Vec::new();
    for node in &legacy.nodes {
        let id = NodeId::mint(nodes::kind(node.new_kind())?, repository.store_mut())?;
        ids.insert(node.id.clone(), id.clone());
        let derive = derived.contains(&node.id);
        nodes.push(nodes::build(node, id, cli.ref_base.as_deref(), derive)?);
    }
    let diagnostics = edges::apply(legacy, &mut nodes, &ids);
    let source = cli.ledger.display().to_string();
    repository.transaction("migrate from 0.4", &source, |repository| {
        for node in &nodes {
            repository.put(node)?;
        }
        Ok(())
    })?;
    Ok((nodes.len(), diagnostics))
}

fn settings(cli: &Cli) {
    if let Some(base) = &cli.ref_base {
        println!("ref base: {base}");
    }
    if let Some(dir) = &cli.publication {
        println!("publication: {}", dir.display());
    }
}
