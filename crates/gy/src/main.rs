//! gy: the ledger CLI. It wires the gy-ledger views and ops to commands and
//! does nothing else (D-76).
mod cli;
mod output;
mod reads;
mod remote_cli;
mod repo;
mod write;
mod writes;

use clap::Parser;
use cli::{Cli, Command, CriterionAction, NeedAction, QuestionAction};
use gy_ledger::{
    CriterionAdd, CriterionSatisfy, NeedAdd, NeedClose, QuestionAdd, QuestionClose, Result, config,
    location, next, reconcile, retry, show,
};
use output::{emit, emit_list, report};
use remote_cli::RemoteAction;
use std::path::Path;
use write::Written;

fn main() {
    let cli = Cli::parse();
    if let Err(error) = dispatch(&cli) {
        report(cli.json, &error);
        std::process::exit(2);
    }
}

/// `init` makes the gy.toml the others need, so it runs before the root is
/// resolved and touches no ledger (n-29b3). Everything else goes to `run`.
/// The old top-level names (`share` / `join` / `sync`) still work and say once,
/// on stderr, to use the `remote` group (n-8d0e); the check comes first, so the
/// warning is the first line even when the command later fails.
fn dispatch(cli: &Cli) -> Result<()> {
    deprecation(&cli.command);
    match &cli.command {
        Command::Init { name } => writes::init(cli, name),
        _ => run(cli),
    }
}

/// The one line an old name prints before it does the same work (n-8d0e).
fn deprecation(command: &Command) {
    let message = match command {
        Command::Share { .. } => "gy share is deprecated; use gy remote set <URL>",
        Command::Join => "gy join is deprecated; use gy remote join",
        Command::Sync => "gy sync is deprecated; use gy remote sync",
        _ => return,
    };
    eprintln!("{message}");
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
    if let Some(outcome) = before_reconcile(cli, &root, &ledger) {
        return outcome;
    }
    prepare(&root, &ledger)?;
    let outcome = match &cli.command {
        Command::Show { ids, full } => {
            let repository = repo::open(&ledger)?;
            emit_list(cli.json, &show(&repository, ids, *full)?)
        }
        Command::List { .. } => reads::read_list(cli, &ledger),
        Command::Next => {
            let repository = repo::open(&ledger)?;
            emit_list(cli.json, &next(&repository, cli.scope.as_deref())?)
        }
        Command::Handover => reads::handover(cli, &root, &ledger),
        Command::Publish { since, out } => {
            reads::write_publish(cli, &root, &ledger, *since, out.as_deref())
        }
        Command::Serve => reads::serve(&root, &ledger, tab_name(&root)),
        Command::Sync => reads::sync_command(cli, &root, &ledger),
        Command::Remote { action } => remote(cli, &root, &ledger, action),
        Command::Init { .. } | Command::Share { .. } | Command::Join => unreachable!("early"),
        Command::Need { action } => write_need(cli, &root, &ledger, action),
        Command::Question { action } => write_question(cli, &root, &ledger, action),
        Command::Criterion { action } => write_criterion(cli, &root, &ledger, action),
        Command::Req { action } => writes::req(cli, &root, &ledger, action),
        Command::Scope { action } => writes::scope(cli, &root, &ledger, action),
        Command::Decide(args) => writes::decide(cli, &root, &ledger, args),
        Command::Link(args) => writes::link(cli, &ledger, args),
        Command::Edit(args) => writes::edit(cli, &root, &ledger, args),
        Command::Undo(args) => writes::undo(cli, &ledger, args),
    };
    // A write to a shared copy starts a detached sync; the write itself waits
    // for nothing (n-ecbf).
    after_write(cli, &root, &ledger, &outcome);
    outcome
}

/// The commands that run before the automatic clone, so their own checks come
/// first and nothing is fetched quietly (n-57c5). `None` is the usual path.
fn before_reconcile(cli: &Cli, root: &Path, ledger: &Path) -> Option<Result<()>> {
    match &cli.command {
        Command::Share { url } => Some(writes::share(cli, root, ledger, url)),
        Command::Join => Some(writes::join(cli, root, ledger)),
        Command::Remote { action } => match action {
            RemoteAction::Set { url } => Some(writes::share(cli, root, ledger, url)),
            RemoteAction::Join => Some(writes::join(cli, root, ledger)),
            RemoteAction::Sync => None,
        },
        _ => None,
    }
}

/// Reconcile the copy's marker and announce refused writes, in one step.
fn prepare(root: &Path, ledger: &Path) -> Result<()> {
    reconcile_copy(root, ledger)?;
    repo::announce_rejected(ledger);
    Ok(())
}

/// Route the `remote` group (n-8d0e): `set` and `join` run before the clone, so
/// only `sync` arrives here.
fn remote(cli: &Cli, root: &Path, ledger: &Path, action: &RemoteAction) -> Result<()> {
    match action {
        RemoteAction::Sync => reads::sync_command(cli, root, ledger),
        RemoteAction::Set { .. } | RemoteAction::Join => unreachable!("early"),
    }
}

/// Reconcile the copy's marker with gy.toml, and tell the reader once when
/// syncing stopped (n-8a52).
fn reconcile_copy(root: &Path, ledger: &Path) -> Result<()> {
    let had = ledger.join("remote").is_file();
    let remote = config::read(root)?.remote;
    if let Some(url) = reconcile(ledger, remote.as_deref())? {
        eprintln!(
            "stopped syncing with {url}; this copy is local from here on and other members' writes will not arrive"
        );
    }
    // The automatic clone (d-b1d4): say once, on stderr, that this machine is
    // now a member. A clone leaves the marker where there was none.
    if !had {
        if let Some(url) = remote.filter(|_| ledger.join("remote").is_file()) {
            eprintln!("{}", gy_ledger::join_notice(ledger, &url));
        }
    }
    Ok(())
}

/// A write to a shared copy starts a detached `gy remote sync` (n-ecbf).
fn after_write(cli: &Cli, root: &Path, ledger: &Path, outcome: &Result<()>) {
    if outcome.is_ok() && writes(&cli.command) {
        // A write on a node a refused write touched clears that notice
        // (n-ecbf 2B2), then a detached sync pushes the write.
        if let Ok((events, _)) = gy_ledger::log::read(ledger) {
            if let Some(event) = events.last() {
                let _ = gy_ledger::clear_rejected(ledger, event);
            }
        }
        let _ = repo::background_sync(root, ledger);
    }
}

/// The commands that change the ledger and may owe the remote a push.
fn writes(command: &Command) -> bool {
    matches!(
        command,
        Command::Need { .. }
            | Command::Question { .. }
            | Command::Criterion { .. }
            | Command::Req { .. }
            | Command::Scope { .. }
            | Command::Decide(_)
            | Command::Link(_)
            | Command::Edit(_)
            | Command::Undo(_)
    )
}

fn write_need(cli: &Cli, root: &Path, ledger: &Path, action: &NeedAction) -> Result<()> {
    let repository = repo::open_write(ledger)?;
    let outcome = match action {
        NeedAction::Add {
            title,
            targets,
            spawned_by,
            body_file,
        } => {
            let operation = NeedAdd {
                scope: write::scope(root, cli.scope.as_deref())?,
                title: title.clone(),
                targets: write::resolve_all(&repository, targets)?,
                spawned_by: write::resolve_opt(&repository, spawned_by.as_deref())?,
                body: write::body_file(body_file.as_deref())?,
            };
            retry(|| repo::open_write(ledger), operation)?.1
        }
        NeedAction::Close { id, by, evidence } => {
            let operation = NeedClose {
                id: repository.resolve(id)?,
                by: write::closed_by(by)?,
                evidence: evidence.clone(),
            };
            retry(|| repo::open_write(ledger), operation)?.1
        }
    };
    emit(
        cli.json,
        &Written::of(&outcome).created(matches!(action, NeedAction::Add { .. })),
    )
}

fn write_question(cli: &Cli, root: &Path, ledger: &Path, action: &QuestionAction) -> Result<()> {
    let repository = repo::open_write(ledger)?;
    let outcome = match action {
        QuestionAction::Add {
            title,
            decider,
            options,
            body_file,
        } => {
            let operation = QuestionAdd {
                scope: write::scope(root, cli.scope.as_deref())?,
                title: title.clone(),
                decider: decider.clone(),
                options: options.clone(),
                body: write::body_file(body_file.as_deref())?,
            };
            retry(|| repo::open_write(ledger), operation)?.1
        }
        QuestionAction::Close {
            id,
            by,
            evidence,
            decision,
        } => {
            let operation = QuestionClose {
                id: repository.resolve(id)?,
                by: write::closure(by)?,
                evidence: evidence.clone(),
                decision: write::resolve_opt(&repository, decision.as_deref())?,
            };
            retry(|| repo::open_write(ledger), operation)?.1
        }
    };
    emit(
        cli.json,
        &Written::of(&outcome).created(matches!(action, QuestionAction::Add { .. })),
    )
}

fn write_criterion(cli: &Cli, root: &Path, ledger: &Path, action: &CriterionAction) -> Result<()> {
    let repository = repo::open_write(ledger)?;
    let outcome = match action {
        CriterionAction::Add { title, body_file } => {
            let operation = CriterionAdd {
                scope: write::scope(root, cli.scope.as_deref())?,
                title: title.clone(),
                body: write::body_file(body_file.as_deref())?,
            };
            retry(|| repo::open_write(ledger), operation)?.1
        }
        CriterionAction::Satisfy {
            id,
            evidence,
            revoke,
        } => {
            let operation = CriterionSatisfy {
                id: repository.resolve(id)?,
                evidence: evidence.clone(),
                revoke: *revoke,
            };
            retry(|| repo::open_write(ledger), operation)?.1
        }
    };
    emit(
        cli.json,
        &Written::of(&outcome).created(matches!(action, CriterionAction::Add { .. })),
    )
}
