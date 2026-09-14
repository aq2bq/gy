use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "gy",
    version,
    about = "gy (good,yes) — a progress-tracking CLI for decisions, questions, and needs in one graph",
    long_about = "gy (good,yes) — checks ledger consistency using YAML frontmatter as the source of truth. It does not call the GitHub API. People and agents must verify that the ledger matches reality and review options and applicability conditions."
)]
pub struct Cli {
    /// Return JSON results. Diagnostics go to stderr; results go to stdout
    #[arg(long, global = true)]
    pub json: bool,
    /// Suppress ordinary output while retaining JSON results and errors
    #[arg(long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
    /// Show additional diagnostics, including the ledger location
    #[arg(long, global = true)]
    pub verbose: bool,
    /// Reads cover all scopes by default. Specify the write scope if it cannot be resolved from the current directory
    #[arg(long, global = true)]
    pub scope: Option<String>,
    /// Search for the ledger starting from this directory
    #[arg(short = 'C', long, global = true)]
    pub cwd: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a ledger and scope, and append instructions to an existing AGENTS.md
    Init {
        name: String,
        #[arg(long)]
        parent_issue: Option<u64>,
    },
    /// Rename a scope and update every member node while preserving identity, relationships, and history
    Scope {
        #[command(subcommand)]
        command: ScopeCommand,
    },
    /// Create needs and record their association with Issues
    Need {
        #[command(subcommand)]
        command: Need,
    },
    /// Create questions and close them in one of three ways
    Question {
        #[command(subcommand)]
        command: Question,
    },
    /// Quick notes for humans only; agents must not use this command. Incomplete records always fail lint
    Q { title: String },
    /// Record a decision. --scope selects the ledger scope; --scope-note states applicable paths and conditions
    Decide {
        title: String,
        #[arg(long)]
        scope_note: Option<String>,
        #[arg(long,action=clap::ArgAction::Append)]
        closes: Vec<String>,
    },
    /// Save relationships on both sides. For narrows / supersedes, --mark identifies the invalidated passage in the older decision
    #[command(
        after_help = "Directions: question closes decision; decision narrows|widens|supersedes|completes decision; need|requirement targets criterion; need spawned-by decision; need filed-as requirement; need depends-on need; requirement relies-on decision; requirement raised question; gate measured-by question\n--mark does not replace the body. show/render annotates the matching text in the older decision. A file-level link alone cannot identify an invalidated sentence."
    )]
    Link {
        source: String,
        label: String,
        target: String,
        #[arg(long)]
        mark: Option<String>,
    },
    /// Register requirements, advance their states, and compress completed records
    Req {
        #[command(subcommand)]
        command: Req,
    },
    /// Register acceptance criteria and record satisfaction
    Criterion {
        #[command(subcommand)]
        command: Criterion,
    },
    /// Register gates for deciding whether an approach should continue
    Gate {
        #[command(subcommand)]
        command: Gate,
    },
    /// Edit arbitrary frontmatter attributes or the body (except IDs and edges)
    Node {
        #[command(subcommand)]
        command: NodeCommand,
    },
    /// Check L1–L14 and inverse links; exit 1 on errors. Incomplete q records fail even with L8/L9 disabled
    Lint,
    /// Generate ledger pages based on node count. Configure output and threshold in gy.toml [render]. --format html writes a single self-contained file
    Render {
        #[arg(long,default_value="markdown",value_parser=["markdown","dot","html"])]
        format: String,
    },
    /// Show a node and relationships on both sides. --graph emits DOT by traversing relationships
    Show {
        id: String,
        #[arg(long)]
        graph: bool,
    },
    /// Search body sections and arbitrary frontmatter attributes across scopes
    #[command(
        after_help = "Example: gy find delivery --where decider=master\n    gy find --where type=decision --where 'created>=2026-09-01'\nOperators: = != >= <= > < ~ (substring). Use paths such as pr.base for nested attributes. Context and applicability conditions are also searched."
    )]
    Find {
        keyword: Option<String>,
        #[arg(long = "where")]
        filters: Vec<String>,
    },
    /// List needs with resolved prerequisites and no unresolved question dependencies, without prioritizing them
    Next,
    /// List lint findings, next-transition evidence, and responsible parties as recorded facts
    Handover,
    /// Report acceptance criterion satisfaction and question arrival rates from git history
    Stats {
        #[arg(long, default_value_t = 7)]
        days: u32,
    },
    /// Print a reference for agents to read at session start
    Cheatsheet,
    /// Generate shell completions to stdout
    Completions { shell: clap_complete::Shell },
    /// Start the MCP server over stdin/stdout
    Mcp {
        #[command(subcommand)]
        command: Mcp,
    },
    /// Install the bundled agent skills
    Skills {
        #[command(subcommand)]
        command: Skills,
    },
    /// Import an ADR directory as decisions, preserving IDs. Reject the entire import on conflicts
    #[command(
        after_help = "Configure [import] scope_note_section in gy.toml to copy an ATX heading section into decision_scope. Existing nonempty decision_scope values take precedence. scope_note_placeholders lists texts to leave unfilled. import_summary reports missing scopes and marks. Frontmatter narrows/supersedes entries receive imported=true; relationships are not inferred from prose."
    )]
    Import { directory: PathBuf },
}
#[derive(Subcommand, Debug)]
pub enum ScopeCommand {
    /// Rename a scope. Node IDs, relationships, records, and history are preserved
    Rename { old: String, new: String },
}
#[derive(Subcommand, Debug)]
pub enum Need {
    Add {
        title: String,
        #[arg(long,action=clap::ArgAction::Append)]
        targets: Vec<String>,
        #[arg(long)]
        spawned_by: Option<String>,
    },
    /// Add filed-as to a requirement whose parent_issue matches the parent Issue in gy.toml
    File {
        id: String,
        #[arg(long)]
        issue: u64,
    },
}
#[derive(Subcommand, Debug)]
pub enum Question {
    /// Search all scopes; review matches before using --force. At least two viable options are required
    Add {
        title: String,
        #[arg(long)]
        decider: Option<String>,
        #[arg(long)]
        options: Vec<String>,
        #[arg(long)]
        bundle: Option<String>,
        #[arg(long)]
        bundle_rationale: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// fact requires --note; decision / non-decision requires --decision. Warn about unresolved references to the closed question
    Close {
        id: String,
        #[arg(long)]
        by: Option<String>,
        #[arg(long)]
        decision: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
}
#[derive(Args, Debug)]
pub struct Advance {
    pub issue: String,
    /// One of 11 states (parenthesized context is allowed). See gy cheatsheet
    #[arg(long)]
    pub to: Option<String>,
    /// Record evidence for the transition so later users can recover its reason
    #[arg(long)]
    pub evidence: Option<String>,
    #[arg(long)]
    pub reported_base: Option<String>,
    #[arg(long)]
    pub reported_files: Option<u64>,
    #[arg(long,action=clap::ArgAction::Set)]
    pub data_migration: Option<bool>,
    #[arg(long,action=clap::ArgAction::Set)]
    pub production_only: Option<bool>,
    #[arg(long,action=clap::ArgAction::Set)]
    pub production_done: Option<bool>,
    #[arg(long,action=clap::ArgAction::Set)]
    pub cleanup_done: Option<bool>,
}
#[derive(Subcommand, Debug)]
pub enum Req {
    /// Register a requirement without querying GitHub. Record the verified parent Issue
    Add {
        title: String,
        #[arg(long)]
        issue: u64,
        #[arg(long)]
        parent_issue: Option<u64>,
    },
    Advance(Advance),
    /// Compress a completed requirement into six items after moving constraints into decisions. Supply the archive comment URL via --evidence
    #[command(
        after_help = "First record summary / contracts_changed / artifacts / production / deviations / residual with node set. Explicitly use 'none' for absent deviations, residual work, or production work.\ncontracts_changed is free-form text or a list. artifacts contains pr / merge_commit / base_branch. Production measurements are retained as arbitrary attributes in production.\nAssociate each constraints text with a decision that has applicability conditions, add relies-on links, and record constraints_reviewed=true. Without --evidence, output the full text for archiving. The caller archives it in the corresponding Issue comment and supplies its URL via --evidence; gy then replaces the body with six items. gy does not query GitHub.\nThe full quality gates, design proposals, and audit records move to the archive. IDs, required attributes, graph relationships, and unknown attributes are retained."
    )]
    Compress {
        issue: String,
        #[arg(long)]
        evidence: Option<String>,
    },
}
#[derive(Subcommand, Debug)]
pub enum Criterion {
    Add {
        title: String,
    },
    /// Record acceptance criterion satisfaction with evidence and prevent the satisfied count from decreasing
    Satisfy {
        id: String,
        #[arg(long)]
        evidence: Option<String>,
    },
}
#[derive(Subcommand, Debug)]
pub enum Gate {
    Add {
        title: String,
        #[arg(long)]
        measured_by: Vec<String>,
    },
}
#[derive(Subcommand, Debug)]
pub enum NodeCommand {
    /// Validate a configured record and save its inputs and schema without changing node state
    Submit {
        id: String,
        #[arg(long)]
        record: String,
        #[arg(long)]
        evidence: String,
    },
    /// Values are JSON, or strings if parsing fails. Use residual for remaining-work destinations and waiting-on for unresolved references
    Set {
        id: String,
        #[arg(long = "set", required_unless_present = "body_file")]
        attributes: Vec<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
}
#[derive(Subcommand, Debug)]
pub enum Mcp {
    Serve,
}
#[derive(Subcommand, Debug)]
pub enum Skills {
    Install { directory: PathBuf },
}
