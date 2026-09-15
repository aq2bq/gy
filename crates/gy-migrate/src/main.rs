//! gy-migrate: read a 0.4 ledger and write it into the new canonical ledger,
//! with its edges, its requirement records frozen as a publication, and the
//! repository's gy.toml (N-66, N-67, N-68, N-69).
mod config;
mod edges;
mod freeze;
mod legacy;
mod nodes;
mod report;

use clap::Parser;
use gy_ledger::{
    Actor, Error, FileStore, FormatVersion, NodeId, Repository, Result, format, location,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    /// Where frozen records and the report are written.
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
    let publication = publication_dir(&cli);
    let mut report = legacy.report();
    report.requirements = requirement_states(&legacy);
    report.publication = Some(publication.display().to_string());
    if cli.dry_run {
        print!("{report}");
        settings(&cli);
        return Ok(());
    }
    let (written, diagnostics, frozen) = write(&cli, &legacy, &publication)?;
    report.frozen = frozen;
    let text = format!("{report}{diagnostics}written: {written} nodes\n");
    print!("{text}");
    write_report(&publication, &text)?;
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

fn publication_dir(cli: &Cli) -> PathBuf {
    cli.publication
        .clone()
        .unwrap_or_else(|| cli.root.join(".gy-migration"))
}

fn requirement_states(legacy: &legacy::Legacy) -> Vec<String> {
    legacy
        .nodes
        .iter()
        .filter(|node| node.kind == "requirement")
        .map(|node| {
            let status = node.status();
            format!("{} {} -> {}", node.id, status, nodes::state(status).name())
        })
        .collect()
}

/// Build every new node, its edges, and its frozen records, then write them in
/// one transaction and generate the repository's gy.toml.
fn write(
    cli: &Cli,
    legacy: &legacy::Legacy,
    publication: &Path,
) -> Result<(usize, edges::Edges, usize)> {
    config::ensure(&cli.root, legacy)?;
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
    let frozen = freeze::freeze(legacy, &mut nodes, publication)?;
    let source = source(cli);
    repository.transaction("migrate from 0.4", &source, |repository| {
        for node in &nodes {
            repository.put(node)?;
        }
        Ok(())
    })?;
    Ok((nodes.len(), diagnostics, frozen))
}

/// The 0.4 directory, with its git commit when it has one.
fn source(cli: &Cli) -> String {
    let directory = cli.ledger.display().to_string();
    match git_commit(&cli.ledger) {
        Some(commit) => format!("{directory} {commit}"),
        None => directory,
    }
}

fn git_commit(dir: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!commit.is_empty()).then_some(commit)
}

fn write_report(publication: &Path, text: &str) -> Result<()> {
    std::fs::create_dir_all(publication)?;
    std::fs::write(publication.join("migration-report.md"), text)?;
    Ok(())
}

fn settings(cli: &Cli) {
    if let Some(base) = &cli.ref_base {
        println!("ref base: {base}");
    }
    if let Some(dir) = &cli.publication {
        println!("publication: {}", dir.display());
    }
}
