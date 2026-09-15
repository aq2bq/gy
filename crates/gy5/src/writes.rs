//! decide, link, edit, and undo: the write commands whose arguments need more
//! than a flat option list.
use crate::Cli;
use crate::output::emit;
use crate::repo;
use crate::write::{self, Written};
use clap::Args;
use gy_ledger::link::Link as LinkOp;
use gy_ledger::{Decide, DecisionScope, Edit, Error, Operation, Relation, Result, Undo};
use std::path::Path;

#[derive(Args)]
pub struct DecideArgs {
    title: String,
    #[arg(long = "scope-note", value_name = "TEXT")]
    scope_note: String,
    #[arg(long = "body-file", value_name = "PATH")]
    body_file: Option<String>,
    #[arg(long, value_name = "Q")]
    closes: Vec<String>,
    #[arg(long, group = "relation", value_name = "D")]
    narrows: Option<String>,
    #[arg(long, group = "relation", value_name = "D")]
    widens: Option<String>,
    #[arg(long, group = "relation", value_name = "D")]
    supersedes: Option<String>,
    #[arg(long, group = "relation", value_name = "D")]
    completes: Option<String>,
    #[arg(long, value_name = "TEXT")]
    mark: Option<String>,
    #[arg(long, value_name = "TEXT")]
    source: Option<String>,
}

#[derive(Args)]
pub struct LinkArgs {
    from: String,
    relation: String,
    to: String,
    #[arg(long, value_name = "TEXT")]
    mark: Option<String>,
    #[arg(long)]
    remove: bool,
}

#[derive(Args)]
pub struct EditArgs {
    id: String,
    #[arg(long, value_name = "TEXT")]
    reason: String,
    #[arg(long, value_name = "TITLE")]
    title: Option<String>,
    #[arg(long = "body-file", value_name = "PATH")]
    body_file: Option<String>,
    #[arg(long, value_name = "KEY=VALUE")]
    set: Vec<String>,
    #[arg(long, value_name = "KEY=VALUE")]
    append: Vec<String>,
}

#[derive(Args)]
pub struct UndoArgs {
    #[arg(long, value_name = "TEXT")]
    reason: String,
}

pub fn decide(cli: &Cli, root: &Path, ledger: &Path, args: &DecideArgs) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let relates = match one_relation(args) {
        Some((relation, text)) => vec![(relation, repository.resolve(text)?, args.mark.clone())],
        None if args.mark.is_some() => return Err(Error::invalid("--mark needs a relation")),
        None => Vec::new(),
    };
    let outcome = Decide {
        scope: write::scope(root, cli.scope.as_deref())?,
        title: args.title.clone(),
        decision_scope: DecisionScope::recorded(args.scope_note.clone())?,
        body: write::body_file(args.body_file.as_deref())?,
        source: args.source.clone(),
        closes: write::resolve_all(&repository, &args.closes)?,
        relates,
    }
    .run(&mut repository)?;
    emit(cli.json, &Written::of(&outcome))
}

/// The single lineage relation decide takes; the four flags are exclusive.
fn one_relation(args: &DecideArgs) -> Option<(Relation, &String)> {
    match (
        &args.narrows,
        &args.widens,
        &args.supersedes,
        &args.completes,
    ) {
        (Some(id), None, None, None) => Some((Relation::Narrows, id)),
        (None, Some(id), None, None) => Some((Relation::Widens, id)),
        (None, None, Some(id), None) => Some((Relation::Supersedes, id)),
        (None, None, None, Some(id)) => Some((Relation::Completes, id)),
        _ => None,
    }
}

pub fn link(cli: &Cli, ledger: &Path, args: &LinkArgs) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let outcome = LinkOp {
        from: repository.resolve(&args.from)?,
        relation: write::relation(&args.relation)?,
        to: repository.resolve(&args.to)?,
        mark: args.mark.clone(),
        remove: args.remove,
    }
    .run(&mut repository)?;
    emit(cli.json, &Written::of(&outcome))
}

pub fn edit(cli: &Cli, ledger: &Path, args: &EditArgs) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let outcome = Edit {
        id: repository.resolve(&args.id)?,
        reason: args.reason.clone(),
        title: args.title.clone(),
        body: write::body_file(args.body_file.as_deref())?,
        set: write::pairs(&args.set)?,
        append: write::pairs(&args.append)?,
    }
    .run(&mut repository)?;
    emit(cli.json, &Written::of(&outcome))
}

pub fn undo(cli: &Cli, ledger: &Path, args: &UndoArgs) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let outcome = Undo {
        reason: args.reason.clone(),
    }
    .run(&mut repository)?;
    emit(cli.json, &Written::of(&outcome))
}
