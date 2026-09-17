//! gy: the ledger CLI. It wires the gy-ledger views and ops to commands and
//! does nothing else (D-76).
mod cli;
mod output;
mod reads;
mod repo;
mod write;
mod writes;

use clap::Parser;
use cli::{Cli, Command, CriterionAction, NeedAction, QuestionAction};
use gy_ledger::{
    CriterionAdd, CriterionSatisfy, NeedAdd, NeedClose, Operation, QuestionAdd, QuestionClose,
    Result, handover, location, next, show,
};
use output::{emit, emit_list, report};
use std::path::Path;
use write::Written;

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(&cli) {
        report(cli.json, &error);
        std::process::exit(2);
    }
}

/// The ledger directory's own name, for the browser tab (n-07f0). A root with
/// no name (the filesystem root) or a name that is not UTF-8 gives none.
fn tab_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string()
}

fn run(cli: &Cli) -> Result<()> {
    let root = repo::root(cli.directory.as_deref())?;
    let ledger = location::ledger_dir(&root);
    match &cli.command {
        Command::Show { ids, full } => {
            let repository = repo::open(&ledger)?;
            emit_list(cli.json, &show(&repository, ids, *full)?)
        }
        Command::List { .. } => reads::read_list(cli, &ledger),
        Command::Next => {
            let repository = repo::open(&ledger)?;
            emit_list(cli.json, &next(&repository, cli.scope.as_deref())?)
        }
        Command::Handover => {
            let repository = repo::open(&ledger)?;
            emit(cli.json, &handover(&repository, cli.scope.as_deref())?)
        }
        Command::Publish { since, out } => {
            reads::write_publish(cli, &root, &ledger, *since, out.as_deref())
        }
        Command::Serve => reads::serve(&ledger, tab_name(&root)),
        Command::Need { action } => write_need(cli, &root, &ledger, action),
        Command::Question { action } => write_question(cli, &root, &ledger, action),
        Command::Criterion { action } => write_criterion(cli, &root, &ledger, action),
        Command::Req { action } => writes::req(cli, &root, &ledger, action),
        Command::Scope { action } => writes::scope(cli, &root, &ledger, action),
        Command::Decide(args) => writes::decide(cli, &root, &ledger, args),
        Command::Link(args) => writes::link(cli, &ledger, args),
        Command::Edit(args) => writes::edit(cli, &root, &ledger, args),
        Command::Undo(args) => writes::undo(cli, &ledger, args),
    }
}

fn write_need(cli: &Cli, root: &Path, ledger: &Path, action: &NeedAction) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let outcome = match action {
        NeedAction::Add {
            title,
            targets,
            spawned_by,
        } => NeedAdd {
            scope: write::scope(root, cli.scope.as_deref())?,
            title: title.clone(),
            targets: write::resolve_all(&repository, targets)?,
            spawned_by: write::resolve_opt(&repository, spawned_by.as_deref())?,
        }
        .run(&mut repository)?,
        NeedAction::Close { id, by, evidence } => NeedClose {
            id: repository.resolve(id)?,
            by: write::closed_by(by)?,
            evidence: evidence.clone(),
        }
        .run(&mut repository)?,
    };
    emit(
        cli.json,
        &Written::of(&outcome).created(matches!(action, NeedAction::Add { .. })),
    )
}

fn write_question(cli: &Cli, root: &Path, ledger: &Path, action: &QuestionAction) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let outcome = match action {
        QuestionAction::Add {
            title,
            decider,
            options,
        } => QuestionAdd {
            scope: write::scope(root, cli.scope.as_deref())?,
            title: title.clone(),
            decider: decider.clone(),
            options: options.clone(),
        }
        .run(&mut repository)?,
        QuestionAction::Close {
            id,
            by,
            evidence,
            decision,
        } => QuestionClose {
            id: repository.resolve(id)?,
            by: write::closure(by)?,
            evidence: evidence.clone(),
            decision: write::resolve_opt(&repository, decision.as_deref())?,
        }
        .run(&mut repository)?,
    };
    emit(
        cli.json,
        &Written::of(&outcome).created(matches!(action, QuestionAction::Add { .. })),
    )
}

fn write_criterion(cli: &Cli, root: &Path, ledger: &Path, action: &CriterionAction) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let outcome = match action {
        CriterionAction::Add { title } => CriterionAdd {
            scope: write::scope(root, cli.scope.as_deref())?,
            title: title.clone(),
        }
        .run(&mut repository)?,
        CriterionAction::Satisfy {
            id,
            evidence,
            revoke,
        } => CriterionSatisfy {
            id: repository.resolve(id)?,
            evidence: evidence.clone(),
            revoke: *revoke,
        }
        .run(&mut repository)?,
    };
    emit(
        cli.json,
        &Written::of(&outcome).created(matches!(action, CriterionAction::Add { .. })),
    )
}
