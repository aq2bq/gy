//! The command line's surface (n-4ce6): the clap definitions in one place. The
//! words here are the words a reader sees in `--help`, so they belong together;
//! main.rs only wires what they name to the views and the ops.
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

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
        /// A write sequence, or a date (YYYY-MM-DD, in your own time zone): that day's start onwards.
        #[arg(long, value_name = "SEQ|DATE")]
        since: Option<String>,
    },
    /// The needs that are ready to work.
    Next,
    /// What a session needs to resume: in-progress requirements and counts.
    Handover,
    /// Write the publication: one file per node under a scope directory.
    Publish {
        /// Include the changes after this write sequence.
        #[arg(long, value_name = "SEQ")]
        since: Option<u64>,
        /// The output directory; defaults to gy.toml's output.
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
    },
    /// Read the ledger in a browser, on 127.0.0.1 until stopped.
    Serve,
    /// Sync this copy with the remote named in gy.toml.
    Sync,
    /// Start sharing: check the remote, write gy.toml, upload the ledger.
    Share {
        #[arg(value_name = "URL")]
        url: String,
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
        action: ReqAction,
    },
    /// Rename a scope: move every node and rewrite gy.toml.
    Scope {
        #[command(subcommand)]
        action: ScopeAction,
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
        #[arg(long = "body-file", value_name = "PATH")]
        body_file: Option<String>,
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
        #[arg(long = "body-file", value_name = "PATH")]
        body_file: Option<String>,
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
    Add {
        title: String,
        #[arg(long = "body-file", value_name = "PATH")]
        body_file: Option<String>,
    },
    /// Record evidence that a criterion holds, or revoke it.
    Satisfy {
        id: String,
        #[arg(long, value_name = "TEXT")]
        evidence: String,
        #[arg(long)]
        revoke: bool,
    },
}

#[derive(Args)]
pub struct DecideArgs {
    pub title: String,
    #[arg(long = "scope-note", value_name = "TEXT")]
    pub scope_note: String,
    #[arg(long = "body-file", value_name = "PATH")]
    pub body_file: Option<String>,
    #[arg(long, value_name = "Q")]
    pub closes: Vec<String>,
    /// One lineage relation and its decision: <relation> <D>, at most once.
    #[arg(long, num_args = 2, value_names = ["RELATION", "D"])]
    pub relate: Vec<String>,
    #[arg(long, value_name = "TEXT")]
    pub mark: Option<String>,
    #[arg(long, value_name = "TEXT")]
    pub source: Option<String>,
}

#[derive(Args)]
pub struct LinkArgs {
    pub from: String,
    pub relation: String,
    pub to: String,
    #[arg(long, value_name = "TEXT")]
    pub mark: Option<String>,
    #[arg(long)]
    pub remove: bool,
}

#[derive(Args)]
pub struct EditArgs {
    pub id: String,
    #[arg(long, value_name = "TEXT")]
    pub reason: String,
    #[arg(long, value_name = "TITLE")]
    pub title: Option<String>,
    #[arg(long = "body-file", value_name = "PATH")]
    pub body_file: Option<String>,
    #[arg(long, value_name = "KEY=VALUE")]
    pub set: Vec<String>,
    #[arg(long, value_name = "KEY=VALUE")]
    pub append: Vec<String>,
}

#[derive(Args)]
pub struct UndoArgs {
    #[arg(long, value_name = "TEXT")]
    pub reason: String,
}

#[derive(Subcommand)]
pub enum ScopeAction {
    /// Move every node of a scope to a new name and rewrite gy.toml.
    Rename { old: String, new: String },
}

#[derive(Subcommand)]
pub enum ReqAction {
    /// File a requirement against needs, decisions, and criteria.
    Add {
        title: String,
        #[arg(long = "need", value_name = "N", required = true)]
        needs: Vec<String>,
        #[arg(long = "relies-on", value_name = "D")]
        relies_on: Vec<String>,
        #[arg(long, value_name = "AC")]
        targets: Vec<String>,
        #[arg(long = "ref", value_name = "REF")]
        reference: Option<String>,
        #[arg(long = "body-file", value_name = "PATH")]
        body_file: Option<String>,
    },
    /// Record the approval that confirms a requirement.
    Approve {
        id: String,
        #[arg(long, value_name = "TEXT")]
        design: String,
        #[arg(long = "heard-by", value_name = "NAME")]
        heard_by: String,
        #[arg(long, value_name = "TEXT")]
        evidence: String,
    },
    /// Send an approved requirement back to filed.
    Revise {
        id: String,
        #[arg(long, value_name = "TEXT")]
        reason: String,
        #[arg(long, value_name = "TEXT")]
        source: String,
    },
    /// Record that an approved requirement shipped.
    Done {
        id: String,
        #[arg(long, value_name = "TEXT")]
        evidence: String,
    },
    /// Cancel a requirement that was not done.
    Cancel {
        id: String,
        #[arg(long, value_name = "TEXT")]
        reason: String,
        #[arg(long, value_name = "TEXT")]
        source: String,
    },
}
