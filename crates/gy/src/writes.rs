//! decide, link, edit, and undo: the write commands whose arguments need more
//! than a flat option list.
use crate::cli::{Cli, DecideArgs, EditArgs, LinkArgs, ReqAction, ScopeAction, UndoArgs};
use crate::output::emit;
use crate::repo;
use crate::write::{self, Written};
use gy_ledger::link::Link as LinkOp;
use gy_ledger::{
    Decide, DecisionScope, Edit, Error, NodeId, Operation, Outcome, Ref, Relation, Repository,
    ReqAdd, ReqApprove, ReqCancel, ReqDone, ReqRevise, Result, ScopeRename, Store, Undo, config,
};
use std::path::Path;

pub fn decide(cli: &Cli, root: &Path, ledger: &Path, args: &DecideArgs) -> Result<()> {
    let mut repository = repo::open_write(ledger)?;
    let relates = match one_relation(args)? {
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

pub fn edit(cli: &Cli, root: &Path, ledger: &Path, args: &EditArgs) -> Result<()> {
    let mut repository = repo::open_write(ledger)?.with_scopes(write::scope_names(root)?);
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

pub fn scope(cli: &Cli, root: &Path, ledger: &Path, action: &ScopeAction) -> Result<()> {
    let mut repository = repo::open_write(ledger)?.with_scopes(write::scope_names(root)?);
    let outcome = match action {
        ScopeAction::Rename { old, new } => {
            let outcome = ScopeRename {
                from: old.clone(),
                to: new.clone(),
            }
            .run(&mut repository)?;
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
    let mut repository = repo::open_write(ledger)?;
    let outcome = apply(&mut repository, root, cli, action)?;
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

fn apply<S: Store>(
    repository: &mut Repository<S>,
    root: &Path,
    cli: &Cli,
    action: &ReqAction,
) -> Result<Outcome<NodeId>> {
    match action {
        ReqAction::Add {
            title,
            needs,
            relies_on,
            targets,
            reference,
        } => ReqAdd {
            scope: write::scope(root, cli.scope.as_deref())?,
            title: title.clone(),
            needs: write::resolve_all(repository, needs)?,
            relies_on: write::resolve_all(repository, relies_on)?,
            targets: write::resolve_all(repository, targets)?,
            reference: reference.clone().map(Ref),
        }
        .run(repository),
        _ => advance(repository, action),
    }
}

fn advance<S: Store>(
    repository: &mut Repository<S>,
    action: &ReqAction,
) -> Result<Outcome<NodeId>> {
    Ok(match action {
        ReqAction::Approve {
            id,
            design,
            heard_by,
            evidence,
        } => ReqApprove {
            id: repository.resolve(id)?,
            design: design.clone(),
            heard_by: heard_by.clone(),
            evidence: evidence.clone(),
        }
        .run(repository)?,
        ReqAction::Revise { id, reason, source } => ReqRevise {
            id: repository.resolve(id)?,
            reason: reason.clone(),
            source: source.clone(),
        }
        .run(repository)?,
        ReqAction::Done { id, evidence } => ReqDone {
            id: repository.resolve(id)?,
            evidence: evidence.clone(),
        }
        .run(repository)?,
        ReqAction::Cancel { id, reason, source } => ReqCancel {
            id: repository.resolve(id)?,
            reason: reason.clone(),
            source: source.clone(),
        }
        .run(repository)?,
        ReqAction::Add { .. } => {
            return Err(Error::invalid("a requirement add is not a transition"));
        }
    })
}
