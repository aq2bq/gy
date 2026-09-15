//! gy-migrate: read a 0.4 ledger and report what it holds. Writing the new
//! nodes into the canonical ledger arrives with the next step (N-67).
mod legacy;

use clap::Parser;
use gy_ledger::{Actor, Error, Result, format, location};
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

fn settings(cli: &Cli) {
    if let Some(base) = &cli.ref_base {
        println!("ref base: {base}");
    }
    if let Some(dir) = &cli.publication {
        println!("publication: {}", dir.display());
    }
}
