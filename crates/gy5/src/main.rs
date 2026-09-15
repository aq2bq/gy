//! gy5: the new gy CLI. It wires the gy-ledger views and ops to commands and
//! does nothing else (D-76).
mod repo;

use clap::{Parser, Subcommand};
use gy_ledger::{Error, Filter, NodeKind, Result, handover, list, location, next, show};
use serde::Serialize;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gy5", version, about = "The new gy ledger")]
struct Cli {
    /// Print the result as JSON (diagnostics go to stderr).
    #[arg(long, global = true)]
    json: bool,
    /// The directory whose gy.toml names the repository (searched upward).
    #[arg(short = 'C', global = true, value_name = "DIR")]
    directory: Option<PathBuf>,
    /// The scope a write uses; required when gy.toml has more than one.
    #[arg(long, global = true, value_name = "NAME")]
    scope: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show one or more nodes by id, alias, or outward reference.
    Show {
        #[arg(required = true, value_name = "ID")]
        ids: Vec<String>,
        /// Show every field, every free attribute, and both edge directions.
        #[arg(long)]
        full: bool,
    },
    /// List node rows, or write units when --actor or --since is given.
    List {
        #[arg(long = "type", value_name = "KIND")]
        kind: Option<String>,
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        #[arg(long, value_name = "ID")]
        targets: Option<String>,
        #[arg(long, value_name = "TEXT")]
        grep: Option<String>,
        #[arg(long, value_name = "NAME")]
        actor: Option<String>,
        #[arg(long, value_name = "SEQ")]
        since: Option<u64>,
    },
    /// The needs that are ready to work.
    Next,
    /// What a session needs to resume: in-progress requirements and counts.
    Handover,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(&cli) {
        report(cli.json, &error);
        std::process::exit(2);
    }
}

fn run(cli: &Cli) -> Result<()> {
    let root = repo::root(cli.directory.as_deref())?;
    let ledger = location::ledger_dir(&root);
    match &cli.command {
        Command::Show { ids, full } => {
            let repository = repo::open(&ledger)?;
            emit_list(cli.json, &show(&repository, ids, *full)?)
        }
        Command::List {
            kind,
            status,
            targets,
            grep,
            actor,
            since,
        } => {
            let repository = repo::open(&ledger)?;
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
        Command::Next => {
            let repository = repo::open(&ledger)?;
            emit_list(cli.json, &next(&repository, cli.scope.as_deref())?)
        }
        Command::Handover => {
            let repository = repo::open(&ledger)?;
            emit(cli.json, &handover(&repository, cli.scope.as_deref())?)
        }
    }
}

fn parse_kind(text: &str) -> Result<NodeKind> {
    NodeKind::ALL
        .into_iter()
        .find(|kind| kind.name() == text || kind.prefix() == text)
        .ok_or_else(|| Error::invalid(format!("unknown type {text}")))
}

fn emit<T: Display + Serialize>(json: bool, value: &T) -> Result<()> {
    if json {
        println!("{}", to_json(value)?);
    } else {
        print!("{value}");
    }
    Ok(())
}

fn emit_list<T: Display + Serialize>(json: bool, values: &[T]) -> Result<()> {
    if json {
        println!("{}", to_json(values)?);
    } else {
        for value in values {
            print!("{value}");
        }
    }
    Ok(())
}

fn to_json<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|error| Error::invalid(error.to_string()))
}

fn report(json: bool, error: &Error) {
    if json {
        println!(
            "{}",
            serde_json::json!({"code": 2, "message": error.message})
        );
    } else {
        eprintln!("{}", error.message);
    }
}
