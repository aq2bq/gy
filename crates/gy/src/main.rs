//! gy: the ledger CLI. It wires the gy-ledger views and ops to commands and
//! does nothing else (D-76).
mod output;
mod reads;
mod repo;
mod write;
mod writes;

use clap::{Parser, Subcommand};
use gy_ledger::{
    CriterionAdd, CriterionSatisfy, NeedAdd, NeedClose, Operation, QuestionAdd, QuestionClose,
    Result, handover, location, next, show,
};
use output::{emit, emit_list, report};
use std::path::{Path, PathBuf};
use write::Written;
use writes::{DecideArgs, EditArgs, LinkArgs, UndoArgs};

#[derive(Parser)]
#[command(name = "gy", version, about = "The gy ledger")]
pub struct Cli {
    /// Print the result as JSON (diagnostics go to stderr).
    #[arg(long, global = true)]
    pub json: bool,
    /// The directory whose gy.toml names the repository (searched upward).
    #[arg(short = 'C', global = true, value_name = "DIR")]
    pub directory: Option<PathBuf>,
    /// The scope a write uses; required when gy.toml has more than one.
    #[arg(long, global = true, value_name = "NAME")]
    pub scope: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
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
    /// Write the record's publication: every node verbatim, the history, and
    /// the diagnostics.
    Publish {
        /// Include the changes after this write sequence.
        #[arg(long, value_name = "SEQ")]
        since: Option<u64>,
        /// Write to this path instead of gy.toml's output or stdout. `{seq}`
        /// is replaced with the write sequence.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// File or close a need.
    Need {
        #[command(subcommand)]
        action: NeedAction,
    },
    /// Open or close a question.
    Question {
        #[command(subcommand)]
        action: QuestionAction,
    },
    /// Add an acceptance criterion, or record it satisfied.
    Criterion {
        #[command(subcommand)]
        action: CriterionAction,
    },
    /// File a requirement, or advance one.
    Req {
        #[command(subcommand)]
        action: writes::ReqAction,
    },
    /// Create a decision, optionally closing questions and linking one relation.
    Decide(DecideArgs),
    /// Add or remove one edge between two nodes.
    Link(LinkArgs),
    /// Change a node's title, body, or free attributes.
    Edit(EditArgs),
    /// Invert the last write as a new transaction.
    Undo(UndoArgs),
}

#[derive(Subcommand)]
pub enum NeedAction {
    /// File a need against existing criteria.
    Add {
        title: String,
        #[arg(long, value_name = "AC", required = true)]
        targets: Vec<String>,
        #[arg(long, value_name = "D")]
        spawned_by: Option<String>,
    },
    /// Close a need by a fact or an external tracker.
    Close {
        id: String,
        #[arg(long, value_name = "KIND")]
        by: String,
        #[arg(long, value_name = "TEXT")]
        evidence: String,
    },
}

#[derive(Subcommand)]
pub enum QuestionAction {
    /// Open a question with a decider and at least two options.
    Add {
        title: String,
        #[arg(long, value_name = "NAME")]
        decider: String,
        #[arg(long = "options", value_name = "OPTION", required = true)]
        options: Vec<String>,
    },
    /// Close a question by a fact, a decision, or neither.
    Close {
        id: String,
        #[arg(long, value_name = "KIND")]
        by: String,
        #[arg(long, value_name = "TEXT")]
        evidence: String,
        #[arg(long, value_name = "D")]
        decision: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CriterionAction {
    /// Add an acceptance criterion.
    Add { title: String },
    /// Record evidence that a criterion holds, or revoke it.
    Satisfy {
        id: String,
        #[arg(long, value_name = "TEXT")]
        evidence: String,
        #[arg(long)]
        revoke: bool,
    },
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
        Command::Need { action } => write_need(cli, &root, &ledger, action),
        Command::Question { action } => write_question(cli, &root, &ledger, action),
        Command::Criterion { action } => write_criterion(cli, &root, &ledger, action),
        Command::Req { action } => writes::req(cli, &root, &ledger, action),
        Command::Decide(args) => writes::decide(cli, &root, &ledger, args),
        Command::Link(args) => writes::link(cli, &ledger, args),
        Command::Edit(args) => writes::edit(cli, &ledger, args),
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
    emit(cli.json, &Written::of(&outcome))
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
    emit(cli.json, &Written::of(&outcome))
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
    emit(cli.json, &Written::of(&outcome))
}
