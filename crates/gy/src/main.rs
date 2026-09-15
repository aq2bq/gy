mod cli;
mod mcp;
use clap::{CommandFactory, Parser};
use cli::*;
use gy_core::*;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

const CHEATSHEET: &str = include_str!("../CHEATSHEET.md");
const SKILLS: &[(&str, &str)] = &[
    ("gy-ledger", include_str!("../skills/gy-ledger/SKILL.md")),
    (
        "gy-question",
        include_str!("../skills/gy-question/SKILL.md"),
    ),
    ("gy-decide", include_str!("../skills/gy-decide/SKILL.md")),
];
struct Output {
    value: Value,
    human: Option<String>,
    warnings: Vec<String>,
    code: u8,
}
impl Output {
    fn new(value: Value) -> Self {
        Self {
            value,
            human: None,
            warnings: vec![],
            code: 0,
        }
    }
    fn text(value: Value, human: String) -> Self {
        Self {
            human: Some(human),
            ..Self::new(value)
        }
    }
}
// Compare only the nodes returned by the operation, not the whole ledger.
fn node_summary(node: &Node, before: &BTreeMap<String, Node>) -> Value {
    let old = before.get(node.id());
    let keys: BTreeSet<_> = node
        .attrs
        .keys()
        .chain(old.into_iter().flat_map(|n| n.attrs.keys()))
        .collect();
    let changed: Vec<_> = keys
        .into_iter()
        .filter(|key| old.and_then(|n| n.attrs.get(*key)) != node.attrs.get(*key))
        .collect();
    json!({"id": node.id(), "type": node.kind(), "scope": node.scope(),
        "changed_attributes": changed,
        "body_changed": old.map(|n| n.body.as_str()).unwrap_or("") != node.body})
}
fn previous_nodes(command: &Commands, store: &Store) -> BTreeMap<String, Node> {
    let ids = match command {
        Commands::Need {
            command: Need::File { id, issue },
        } => vec![id.clone(), issue_id(&issue.to_string())],
        Commands::Question {
            command: Question::Close { id, .. },
        }
        | Commands::Criterion {
            command: Criterion::Satisfy { id, .. },
        }
        | Commands::Node {
            command: NodeCommand::Set { id, .. },
        }
        | Commands::Node {
            command: NodeCommand::Submit { id, .. },
        } => vec![id.clone()],
        Commands::Link { source, target, .. } => vec![source.clone(), target.clone()],
        Commands::Req {
            command: Req::Advance(a),
        } => vec![issue_id(&a.issue)],
        Commands::Req {
            command:
                Req::Compress {
                    issue,
                    evidence: Some(_),
                },
        } => vec![issue_id(issue)],
        _ => vec![],
    };
    ids.into_iter()
        .filter_map(|id| store.nodes.get(&id).cloned().map(|n| (id, n)))
        .collect()
}
fn issue_id(s: &str) -> String {
    if s.starts_with('#') {
        s.into()
    } else {
        format!("#{s}")
    }
}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let json_mode = args.iter().any(|s| s == "--json");
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            if json_mode {
                let value = json!({"code":code,"message":e.to_string()});
                if code == 0 {
                    println!("{value}");
                } else {
                    eprintln!("{value}");
                }
            } else {
                let _ = e.print();
            }
            std::process::exit(code);
        }
    };
    if matches!(cli.command, Commands::Mcp { .. }) {
        if let Err(e) = mcp::serve(&cli) {
            eprintln!("{}", e.message);
            std::process::exit(e.code.into());
        }
        return;
    }
    match execute(&cli) {
        Ok(output) => {
            for warning in &output.warnings {
                if cli.json {
                    eprintln!("{}", json!({"severity":"warn","message":warning}));
                } else {
                    eprintln!("Warning: {warning}");
                }
            }
            if cli.json {
                println!("{}", output.value);
            } else if !cli.quiet {
                if let Some(human) = output.human {
                    print!("{human}");
                    if !human.ends_with('\n') {
                        println!();
                    }
                } else {
                    println!("{}", serde_json::to_string_pretty(&output.value).unwrap());
                }
            }
            std::process::exit(output.code.into());
        }
        Err(e) => {
            if cli.json {
                eprintln!("{}", serde_json::to_string(&e).unwrap());
            } else {
                eprintln!("error: {}", e.message);
            }
            std::process::exit(e.code.into());
        }
    }
}
fn execute(cli: &Cli) -> Result<Output> {
    let cwd = cli.cwd.clone().unwrap_or(std::env::current_dir()?);
    match &cli.command {
        Commands::Cheatsheet => {
            return Ok(Output::text(
                json!({"cheatsheet":CHEATSHEET}),
                CHEATSHEET.into(),
            ));
        }
        Commands::Completions { shell } => {
            let mut buffer = vec![];
            clap_complete::generate(*shell, &mut Cli::command(), "gy", &mut buffer);
            let script = String::from_utf8(buffer).unwrap();
            return Ok(Output::text(
                json!({"shell":shell.to_string(),"script":script}),
                script,
            ));
        }
        Commands::Skills {
            command: Skills::Install { directory },
        } => {
            let directory = cwd.join(directory);
            // Check every destination before writing any skill.
            for (name, body) in SKILLS {
                let path = directory.join(name).join("SKILL.md");
                if path.exists() && fs::read_to_string(&path)? != *body {
                    return Err(Error::input(format!(
                        "{} contains an edited skill. Specify a different destination",
                        path.display()
                    )));
                }
            }
            let mut paths = vec![];
            for (name, body) in SKILLS {
                let path = directory.join(name).join("SKILL.md");
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, body)?;
                paths.push(path);
            }
            return Ok(Output::new(json!({"installed":paths})));
        }
        Commands::Init { name, parent_issue } => {
            let store = Store::init(&cwd, name, *parent_issue)?;
            let value = json!({"root":store.root,"scope":name});
            return Ok(Output::text(value.clone(), value.to_string()));
        }
        Commands::Scope { command } => {
            let mut store = Store::open(&cwd)?;
            let ScopeCommand::Rename { old, new } = command;
            let before: BTreeMap<_, _> = store
                .nodes
                .iter()
                .filter(|(_, n)| n.scope() == old)
                .map(|(id, n)| (id.clone(), n.clone()))
                .collect();
            store.rename_scope(old, new)?;
            let nodes: Vec<_> = before
                .keys()
                .map(|id| node_summary(&store.nodes[id], &before))
                .collect();
            let value = json!({"nodes": nodes});
            return Ok(Output::text(value.clone(), value.to_string()));
        }
        Commands::Mcp { .. } => return Err(Error::input("Cannot start MCP recursively")),
        _ => {}
    }
    let mut store = Store::open(&cwd)?;
    if cli.verbose {
        eprintln!(
            "Ledger: {} ({} nodes)",
            store.root.display(),
            store.nodes.len()
        );
    }
    if let Some(scope) = &cli.scope {
        store.scope(Some(scope))?;
    }
    let scope = cli.scope.as_deref();
    let before = previous_nodes(&cli.command, &store);
    let mut output = Output::new(Value::Null);
    let mut write = true;
    match &cli.command {
        Commands::Need { command } => match command {
            Need::Add {
                title,
                targets,
                spawned_by,
            } => {
                let s = store.scope(scope)?;
                let id = store.add_need(&s, title, targets, spawned_by.as_deref())?;
                output.value = json!({"node":node_summary(store.node(&id)?, &before)});
            }
            Need::File { id, issue } => {
                let req = store.file_need(id, *issue)?;
                output.value = json!({"need":node_summary(store.node(id)?, &before),"requirement":node_summary(store.node(&req)?, &before)});
            }
        },
        Commands::Question { command } => match command {
            Question::Add {
                title,
                decider,
                options,
                bundle,
                bundle_rationale,
                force,
            } => {
                let s = store.scope(scope)?;
                let (id, warnings) = store.add_question(
                    &s,
                    title,
                    QuestionOptions {
                        decider: decider.as_deref().unwrap_or(""),
                        options,
                        bundle: bundle.as_deref(),
                        rationale: bundle_rationale.as_deref(),
                        force: *force,
                    },
                )?;
                output.value =
                    json!({"node":node_summary(store.node(&id)?, &before),"search_hits":warnings});
                output.warnings = warnings;
            }
            Question::Close {
                id,
                by,
                decision,
                note,
            } => {
                output.warnings = store.close_question(
                    id,
                    by.as_deref().unwrap_or(""),
                    decision.as_deref(),
                    note.as_deref(),
                )?;
                output.value = json!({"node":node_summary(store.node(id)?, &before),"warnings":output.warnings});
            }
        },
        Commands::Q { title } => {
            let s = store.scope(scope)?;
            let id = store.create("question", title, &s, None)?;
            let n = store.nodes.get_mut(&id).unwrap();
            n.put("status", "open");
            n.put("capture", true);
            output.value = json!({"node":node_summary(n, &before)});
            output.warnings.push("Recorded an incomplete human note. lint will fail until decider and options are supplied".into());
        }
        Commands::Decide {
            title,
            scope_note,
            closes,
        } => {
            let s = store.scope(scope)?;
            let (id, warnings) =
                store.decide(&s, title, scope_note.as_deref().unwrap_or(""), closes)?;
            output.value =
                json!({"node":node_summary(store.node(&id)?, &before),"open_questions":warnings});
            output.warnings = warnings;
        }
        Commands::Link {
            source,
            label,
            target,
            mark,
        } => {
            if label == "closes" {
                output.warnings = store.close_question(source, "decision", Some(target), None)?;
            } else if label == "filed-as" {
                let issue = target
                    .trim_start_matches('#')
                    .parse()
                    .map_err(|_| Error::input("A filed-as target must be #<Issue-number>"))?;
                store.file_need(source, issue)?;
            } else {
                output.warnings = store.link(source, label, target, mark.as_deref())?;
            }
            output.value = json!({"source":node_summary(store.node(source)?, &before),"target":node_summary(store.node(target)?, &before),"warnings":output.warnings});
        }
        Commands::Req { command } => match command {
            Req::Add {
                title,
                issue,
                parent_issue,
            } => {
                let s = store.scope(scope)?;
                let id = store.create("requirement", title, &s, Some(*issue))?;
                let n = store.nodes.get_mut(&id).unwrap();
                n.put("status", "defining");
                if let Some(p) = parent_issue {
                    n.put("parent_issue", p);
                }
                output.value = json!({"node":node_summary(n, &before)});
            }
            Req::Advance(a) => {
                let id = issue_id(&a.issue);
                store.advance(
                    &id,
                    AdvanceOptions {
                        to: a.to.as_deref().unwrap_or(""),
                        evidence: a.evidence.as_deref().unwrap_or(""),
                        reported_base: a.reported_base.as_deref(),
                        reported_files: a.reported_files,
                        data_migration: a.data_migration,
                        production_only: a.production_only,
                        production_done: a.production_done,
                        cleanup_done: a.cleanup_done,
                    },
                )?;
                output.value = json!({"node":node_summary(store.node(&id)?, &before)});
            }
            Req::Compress { issue, evidence } => {
                let id = issue_id(issue);
                if let Some(evidence) = evidence {
                    store.compress(&id, evidence)?;
                    output.value = json!({"node":node_summary(store.node(&id)?, &before)});
                } else {
                    write = false;
                    let original = store.compression_preview(&id)?;
                    output = Output::text(
                        json!({"id":id,"archive":original,"next":"Archive the full text in an Issue comment, then compress with --evidence <URL>"}),
                        original,
                    );
                }
            }
        },
        Commands::Criterion { command } => match command {
            Criterion::Add { title } => {
                let s = store.scope(scope)?;
                let id = store.create("criterion", title, &s, None)?;
                output.value = json!({"node":node_summary(store.node(&id)?, &before)});
            }
            Criterion::Satisfy { id, evidence } => {
                store.typed(id, "criterion")?;
                let evidence = evidence.as_deref().filter(|s| !s.trim().is_empty()).ok_or_else(|| Error::input("--evidence is required. Record the evidence for judging the acceptance criterion satisfied"))?;
                let n = store.nodes.get_mut(id).unwrap();
                if n.attrs.get("satisfied") != Some(&json!(true)) {
                    n.put("satisfied", true);
                    n.put("satisfied_at", gy_core::today());
                    n.put("evidence", evidence);
                }
                output.value = json!({"node":node_summary(n, &before)});
            }
        },
        Commands::Gate {
            command: Gate::Add { title, measured_by },
        } => {
            let s = store.scope(scope)?;
            for q in measured_by {
                store.typed(q, "question")?;
            }
            let id = store.create("gate", title, &s, None)?;
            for q in measured_by {
                store.link(&id, "measured-by", q, None)?;
            }
            output.value = json!({"node":node_summary(store.node(&id)?, &before)});
        }
        Commands::Node {
            command:
                NodeCommand::Set {
                    id,
                    attributes,
                    body_file,
                },
        } => {
            let mut attrs = serde_json::Map::new();
            for a in attributes {
                let (k, v) = a
                    .split_once('=')
                    .ok_or_else(|| Error::input("Use --set key=value"))?;
                if k.trim().is_empty() {
                    return Err(Error::input("The attribute name is empty"));
                }
                attrs.insert(
                    k.into(),
                    serde_json::from_str(v).unwrap_or_else(|_| json!(v)),
                );
            }
            let n = store.node(id)?;
            if n.kind() == "criterion"
                && n.attrs.get("satisfied") == Some(&json!(true))
                && attrs.get("satisfied").is_some_and(|v| v != &json!(true))
            {
                return Err(Error::input(
                    "A satisfied acceptance criterion cannot be reverted. Register the changed criterion as a new node",
                ));
            }
            store.set_attributes(
                id,
                &attrs,
                body_file
                    .as_ref()
                    .map(|p| fs::read_to_string(cwd.join(p)))
                    .transpose()?,
            )?;
            output.value = json!({"node":node_summary(store.node(id)?, &before)});
        }
        Commands::Node {
            command:
                NodeCommand::Submit {
                    id,
                    record,
                    evidence,
                },
        } => {
            store.submit_record(id, record, evidence)?;
            let schema = &store.config.workflow.records[record];
            let revision = schema
                .version_field
                .as_ref()
                .and_then(|field| store.nodes[id].attrs[record].get(field));
            output.value = json!({"node": node_summary(store.node(id)?, &before),
                "submission": {"record": record, "revision": revision}});
        }
        Commands::Lint => {
            write = false;
            let ds = store.lint(scope);
            output.code = u8::from(ds.iter().any(|d| d.severity == "error"));
            let human = format!(
                "{}\n{}\n",
                if ds.is_empty() {
                    "lint: no findings (L1–L14, inverse links, and configured workflow)".into()
                } else {
                    ds.iter()
                        .map(|d| format!("{} {} {}: {}", d.severity, d.rule, d.id, d.message))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
                INTEGRITY_NOTE
            );
            output.human = Some(human);
            output.value = json!({"diagnostics":ds,"note":INTEGRITY_NOTE});
        }
        Commands::Show { id, graph } => {
            write = false;
            let n = store.node(id)?;
            if scope.is_some_and(|s| n.scope() != s) {
                return Err(Error::input(format!("{id} is not in the specified scope")));
            }
            if *graph {
                let dot = store.dot(scope, Some(id))?;
                output = Output::text(json!({"dot":dot}), dot);
            } else {
                let display = store.display_node(n)?;
                let neighbors = store.neighbors(id)?;
                output = Output::text(
                    json!({"node":n,"neighbors":neighbors,"display":display,"decision_dependencies":store.decision_dependencies(n)}),
                    format!(
                        "{display}\nRelationships:\n{}",
                        serde_json::to_string_pretty(&neighbors).unwrap()
                    ),
                );
            }
        }
        Commands::Find { keyword, filters } => {
            write = false;
            let hits = store.find(keyword.as_deref(), filters, scope)?;
            let human = hits
                .iter()
                .map(|h| format!("{} ({}) [{}] {}", h.id, h.scope, h.section, h.excerpt))
                .collect::<Vec<_>>()
                .join("\n");
            output = Output::text(json!({"hits":hits}), human);
        }
        Commands::Next => {
            write = false;
            let nodes = store.next(scope);
            output = Output::text(
                json!({"nodes":nodes}),
                nodes
                    .iter()
                    .map(|n| format!("{} ({}) {}", n.id(), n.scope(), n.get("title")))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        Commands::Handover => {
            write = false;
            output.value = store.handover(scope);
            output.code = u8::from(
                output.value["lint"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|d| d["severity"] == "error")
                    || !output.value["missing"].as_array().unwrap().is_empty()
                    || !output.value["dangling"].as_array().unwrap().is_empty(),
            );
        }
        Commands::Stats { days } => {
            write = false;
            output.value = store.stats(scope, *days)?;
        }
        Commands::Import { directory } => {
            let s = store.scope(scope)?;
            let imported = store.import_adr(&cwd.join(directory), &s)?;
            let mut missing_scope = vec![];
            let mut missing_marks = vec![];
            for id in &imported {
                let node = store.node(id)?;
                if node.get("decision_scope").trim().is_empty() {
                    missing_scope.push(id.clone());
                }
                for relationship in ["narrows", "supersedes"] {
                    for target in node.refs(relationship) {
                        if gy_core::edge_mark(node, relationship, &target).is_none() {
                            missing_marks.push(json!({"source": id, "relationship": relationship, "target": target}));
                        }
                    }
                }
            }
            output.warnings.push(format!(
                "Imported {} decisions; {} missing decision_scope; {} relationships missing mark. Review import_summary and run gy lint.",
                imported.len(), missing_scope.len(), missing_marks.len()
            ));
            output.value = json!({
                "imported": imported.iter().map(|id| node_summary(&store.nodes[id], &before)).collect::<Vec<_>>(),
                "import_summary": {
                    "imported_count": imported.len(),
                    "missing_decision_scope_count": missing_scope.len(),
                    "missing_decision_scope": missing_scope,
                    "missing_mark_count": missing_marks.len(),
                    "missing_marks": missing_marks
                }
            });
        }
        _ => unreachable!(),
    }
    if write {
        output.human = Some(output.value.to_string());
        store.commit()?;
    }
    Ok(output)
}
