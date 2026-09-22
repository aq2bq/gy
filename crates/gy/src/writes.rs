//! decide, link, edit, and undo: the write commands whose arguments need more
//! than a flat option list. Every write goes through `retry`, so a lost race
//! with another writer is tried again on the ledger as it then stands (n-fe59).
use crate::cli::{Cli, DecideArgs, EditArgs, LinkArgs, ReqAction, ScopeAction, UndoArgs};
use crate::output::emit;
use crate::repo;
use crate::write::{self, Written};
use gy_ledger::link::Link as LinkOp;
use gy_ledger::{
    Decide, DecisionScope, Edit, Error, FileStore, NodeId, Outcome, Ref, Relation, Repository,
    ReqAdd, ReqApprove, ReqCancel, ReqDone, ReqRevise, Result, Rules, ScopeRename, Share, Store,
    Undo, config, join_check, join_run, retry, share_check, share_upload,
};
use std::path::Path;
use std::sync::Arc;

/// `gy init <name>`: start a repository here, or report the one already here
/// (n-29b3). It runs before the root is resolved, so it never opens the ledger,
/// reconciles a copy, or syncs.
pub fn init(cli: &Cli, name: &str) -> Result<()> {
    config::check_scope_name(name)?;
    match repo::find_root(cli.directory.as_deref())? {
        Some(root) => {
            let settings = config::read(&root)?;
            emit(cli.json, &config::Init::existing(&root, &settings))
        }
        None => {
            let root = repo::init_dir(cli.directory.as_deref())?;
            config::init(&root, name)?;
            emit(cli.json, &config::Init::created(&root, name))
        }
    }
}

/// `gy remote join`: check everything, then take the copy and say who writes
/// (n-57c5, ac-545c). The gy.toml remote is required; the automatic clone is
/// for the other commands.
pub fn join(cli: &Cli, root: &Path, ledger: &Path) -> Result<()> {
    let remote = config::read(root)?.remote.ok_or_else(|| {
        Error::invalid("this project is not shared; the owner runs gy remote set <URL>")
    })?;
    join_check(root, &remote)?;
    let actor = std::env::var("GY_ACTOR").ok();
    emit(cli.json, &join_run(ledger, &remote, actor.as_deref())?)
}

/// `gy remote set <URL>`: check the remote, write gy.toml, upload the ledger
/// (n-57c5, ac-efd7). The order lives here: the checks and the words are in
/// `gy-ledger`, the gy.toml write in `ops::config`, and store cannot call ops.
pub fn share(cli: &Cli, root: &Path, ledger: &Path, url: &str) -> Result<()> {
    match config::read(root)?.remote {
        Some(existing) if existing == url => {
            if ledger.join("remote").is_file() {
                return emit(cli.json, &Share::already(url));
            }
            return Err(Error::invalid("this project is shared; run gy remote join"));
        }
        Some(existing) => {
            return Err(Error::invalid(format!(
                "this project is shared with {existing}; gy does not switch remotes"
            )));
        }
        None => {}
    }
    let checked = share_check(root, url)?;
    config::write_remote(root, url)?;
    let uploaded = share_upload(ledger, url, &checked.branch, Arc::new(Rules))?;
    emit(cli.json, &Share::shared(url, checked.line, uploaded))
}

pub fn decide(cli: &Cli, root: &Path, ledger: &Path, args: &DecideArgs) -> Result<()> {
    let repository = repo::open_write(ledger)?;
    let relates = match one_relation(args)? {
        Some((relation, text)) => vec![(relation, repository.resolve(text)?, args.mark.clone())],
        None if args.mark.is_some() => return Err(Error::invalid("--mark needs a relation")),
        None => Vec::new(),
    };
    let operation = Decide {
        scope: write::scope(root, cli.scope.as_deref())?,
        title: args.title.clone(),
        decision_scope: DecisionScope::recorded(args.scope_note.clone())?,
        body: write::body_file(args.body_file.as_deref())?,
        source: args.source.clone(),
        closes: write::resolve_all(&repository, &args.closes)?,
        relates,
    };
    let (_, outcome) = retry(|| repo::open_write(ledger), operation)?;
    emit(cli.json, &Written::of(&outcome).created(true))
}

/// The single lineage relation decide takes: a canonical name and a decision,
/// once. `link` carries any second relation.
fn one_relation(args: &DecideArgs) -> Result<Option<(Relation, &String)>> {
    match args.relate.as_slice() {
        [] => Ok(None),
        [name, id] => {
            let relation = write::relation(name)?;
            if !matches!(
                relation,
                Relation::Narrows | Relation::Widens | Relation::Supersedes | Relation::Completes
            ) {
                return Err(Error::invalid(format!("{name} is not a lineage relation")));
            }
            Ok(Some((relation, id)))
        }
        _ => Err(Error::invalid(
            "--relate takes one relation and one decision",
        )),
    }
}

pub fn link(cli: &Cli, ledger: &Path, args: &LinkArgs) -> Result<()> {
    let repository = repo::open_write(ledger)?;
    let operation = LinkOp {
        from: repository.resolve(&args.from)?,
        relation: write::relation(&args.relation)?,
        to: repository.resolve(&args.to)?,
        mark: args.mark.clone(),
        remove: args.remove,
    };
    let (_, outcome) = retry(|| repo::open_write(ledger), operation)?;
    emit(cli.json, &Written::of(&outcome))
}

pub fn edit(cli: &Cli, root: &Path, ledger: &Path, args: &EditArgs) -> Result<()> {
    let scopes = write::scope_names(root)?;
    let repository = repo::open_write(ledger)?.with_scopes(scopes.clone());
    let operation = Edit {
        id: repository.resolve(&args.id)?,
        reason: args.reason.clone(),
        title: args.title.clone(),
        body: write::body_file(args.body_file.as_deref())?,
        set: write::pairs(&args.set)?,
        append: write::pairs(&args.append)?,
    };
    let open =
        move || repo::open_write(ledger).map(|repository| repository.with_scopes(scopes.clone()));
    let (_, outcome) = retry(open, operation)?;
    emit(cli.json, &Written::of(&outcome))
}

pub fn undo(cli: &Cli, ledger: &Path, args: &UndoArgs) -> Result<()> {
    let operation = Undo {
        reason: args.reason.clone(),
    };
    let (_, outcome) = retry(|| repo::open_write(ledger), operation)?;
    emit(cli.json, &Written::of(&outcome))
}

pub fn scope(cli: &Cli, root: &Path, ledger: &Path, action: &ScopeAction) -> Result<()> {
    let scopes = write::scope_names(root)?;
    let open =
        move || repo::open_write(ledger).map(|repository| repository.with_scopes(scopes.clone()));
    let outcome = match action {
        ScopeAction::Rename { old, new } => {
            let operation = ScopeRename {
                from: old.clone(),
                to: new.clone(),
            };
            let (mut repository, outcome) = retry(open, operation)?;
            // The log is the source of truth; if gy.toml cannot follow, put it
            // back (n-24dd).
            if let Err(error) = config::rename_scope(root, old, new) {
                let _ = repository
                    .store_mut()
                    .undo("scope rename rollback", "scope rename");
                return Err(error);
            }
            outcome
        }
    };
    emit(cli.json, &Written::of(&outcome))
}

pub fn req(cli: &Cli, root: &Path, ledger: &Path, action: &ReqAction) -> Result<()> {
    let (repository, outcome) = run_req(cli, root, ledger, action)?;
    let reference = match &outcome.id {
        Some(id) => write::requirement_reference(&repository, id)?,
        None => None,
    };
    emit(
        cli.json,
        &Written::of(&outcome)
            .created(matches!(action, ReqAction::Add { .. }))
            .reference(reference),
    )
}

type ReqRun = (Repository<FileStore>, Outcome<NodeId>);

fn run_req(cli: &Cli, root: &Path, ledger: &Path, action: &ReqAction) -> Result<ReqRun> {
    let repository = repo::open_write(ledger)?;
    match action {
        ReqAction::Add { .. } => add(cli, root, ledger, &repository, action),
        ReqAction::Approve { .. } => approve(ledger, &repository, action),
        ReqAction::Revise { .. } => revise(ledger, &repository, action),
        ReqAction::Done { .. } => done(ledger, &repository, action),
        ReqAction::Cancel { .. } => cancel(ledger, &repository, action),
    }
}

fn add(
    cli: &Cli,
    root: &Path,
    ledger: &Path,
    repository: &Repository<FileStore>,
    action: &ReqAction,
) -> Result<ReqRun> {
    let ReqAction::Add {
        title,
        needs,
        relies_on,
        targets,
        reference,
        body_file,
    } = action
    else {
        unreachable!("add is a requirement add")
    };
    let operation = ReqAdd {
        scope: write::scope(root, cli.scope.as_deref())?,
        title: title.clone(),
        needs: write::resolve_all(repository, needs)?,
        relies_on: write::resolve_all(repository, relies_on)?,
        targets: write::resolve_all(repository, targets)?,
        reference: reference.clone().map(Ref),
        body: write::body_file(body_file.as_deref())?,
    };
    retry(|| repo::open_write(ledger), operation)
}

fn approve(
    ledger: &Path,
    repository: &Repository<FileStore>,
    action: &ReqAction,
) -> Result<ReqRun> {
    let ReqAction::Approve {
        id,
        design,
        heard_by,
        evidence,
    } = action
    else {
        unreachable!("approve is a requirement approve")
    };
    let operation = ReqApprove {
        id: repository.resolve(id)?,
        design: design.clone(),
        heard_by: heard_by.clone(),
        evidence: evidence.clone(),
    };
    retry(|| repo::open_write(ledger), operation)
}

fn revise(ledger: &Path, repository: &Repository<FileStore>, action: &ReqAction) -> Result<ReqRun> {
    let ReqAction::Revise { id, reason, source } = action else {
        unreachable!("revise is a requirement revise")
    };
    let operation = ReqRevise {
        id: repository.resolve(id)?,
        reason: reason.clone(),
        source: source.clone(),
    };
    retry(|| repo::open_write(ledger), operation)
}

fn done(ledger: &Path, repository: &Repository<FileStore>, action: &ReqAction) -> Result<ReqRun> {
    let ReqAction::Done { id, evidence } = action else {
        unreachable!("done is a requirement done")
    };
    let operation = ReqDone {
        id: repository.resolve(id)?,
        evidence: evidence.clone(),
    };
    retry(|| repo::open_write(ledger), operation)
}

fn cancel(ledger: &Path, repository: &Repository<FileStore>, action: &ReqAction) -> Result<ReqRun> {
    let ReqAction::Cancel { id, reason, source } = action else {
        unreachable!("cancel is a requirement cancel")
    };
    let operation = ReqCancel {
        id: repository.resolve(id)?,
        reason: reason.clone(),
        source: source.clone(),
    };
    retry(|| repo::open_write(ledger), operation)
}
