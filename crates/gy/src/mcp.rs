use crate::{Cli, Commands, Error, Result, execute};
use clap::Parser;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

const TOOL_NAMES: &[(&str, &str, &[&str])] = &[
    (
        "gy_find",
        "Search sections and arbitrary attributes across all scopes",
        &["find"],
    ),
    (
        "gy_show",
        "Show a node and its surrounding relationships",
        &["show"],
    ),
    ("gy_lint", "Check internal ledger consistency", &["lint"]),
    (
        "gy_next",
        "List needs whose prerequisites are resolved",
        &["next"],
    ),
    (
        "gy_stats",
        "Report acceptance criteria and question arrival rates",
        &["stats"],
    ),
    (
        "gy_handover",
        "List records needed for a handover",
        &["handover"],
    ),
    ("gy_need", "Create needs and record their filing", &["need"]),
    ("gy_question", "Create and close questions", &["question"]),
    (
        "gy_decide",
        "Record a decision with applicability conditions",
        &["decide"],
    ),
    ("gy_link", "Update relationships on both sides", &["link"]),
    (
        "gy_req",
        "Register requirements, advance states, and compress records",
        &["req"],
    ),
    (
        "gy_criterion",
        "Register and satisfy acceptance criteria",
        &["criterion"],
    ),
    ("gy_gate", "Register continuation gates", &["gate"]),
    (
        "gy_node",
        "Edit attributes and body text, or validate and submit a configured record",
        &["node"],
    ),
    (
        "gy_render",
        "Generate Markdown, DOT, or a single-file HTML view",
        &["render"],
    ),
    (
        "gy_import",
        "Import existing ADRs while preserving IDs",
        &["import"],
    ),
    ("gy_init", "Create a ledger and scope", &["init"]),
    (
        "gy_scope",
        "Rename a scope while preserving node identity, relationships, records, and history",
        &["scope"],
    ),
    (
        "gy_cheatsheet",
        "Usage instructions for the first session",
        &["cheatsheet"],
    ),
];
pub fn serve(parent: &Cli) -> Result<()> {
    let input = io::stdin();
    let mut output = io::stdout().lock();
    for line in input.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                writeln!(
                    output,
                    "{}",
                    json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}})
                )?;
                output.flush()?;
                continue;
            }
        };
        let Some(id) = request.get("id") else {
            continue;
        };
        let response = handle(&request, parent);
        let response = match response {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err((code, message)) => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
            }
        };
        writeln!(output, "{response}")?;
        output.flush()?;
    }
    Ok(())
}
fn handle(request: &Value, parent: &Cli) -> std::result::Result<Value, (i32, String)> {
    if request["jsonrpc"] != "2.0" {
        return Err((-32600, "JSON-RPC 2.0 required".into()));
    }
    match request["method"].as_str().unwrap_or("") {
        "initialize" => {
            let requested = request["params"]["protocolVersion"].as_str().unwrap_or("");
            let version =
                if ["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"].contains(&requested) {
                    requested
                } else {
                    "2025-11-25"
                };
            Ok(
                json!({"protocolVersion":version,"capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"gy","version":env!("CARGO_PKG_VERSION")},"instructions":"Call gy_cheatsheet first. args is an array of arguments after the corresponding CLI subcommand, not a shell string. Verify separately that the ledger matches reality."}),
            )
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(
            json!({"tools":TOOL_NAMES.iter().map(|(name,description,_)|json!({"name":name,"description":description,"inputSchema":{"type":"object","properties":{"args":{"type":"array","items":{"type":"string"},"description":"CLI arguments. Example for question: [\"add\",\"Question\",\"--decider\",\"master\",\"--options\",\"A\",\"--options\",\"B\",\"--scope\",\"demo\"]"}},"additionalProperties":false}})).collect::<Vec<_>>()}),
        ),
        "tools/call" => {
            let name = request["params"]["name"].as_str().unwrap_or("");
            let (_, _, prefix) = TOOL_NAMES
                .iter()
                .find(|t| t.0 == name)
                .ok_or_else(|| (-32602, format!("Unknown tool {name}")))?;
            let arguments = &request["params"]["arguments"];
            if !arguments.is_null() && !arguments.is_object() {
                return Err((-32602, "arguments must be an object".into()));
            }
            if arguments
                .as_object()
                .is_some_and(|m| m.keys().any(|k| k != "args"))
            {
                return Err((-32602, "Only args is accepted".into()));
            }
            let mut args = vec!["gy".to_string(), "--json".to_string()];
            if let Some(cwd) = &parent.cwd {
                args.extend(["--cwd".into(), cwd.to_string_lossy().into()]);
            }
            if let Some(scope) = &parent.scope {
                args.extend(["--scope".into(), scope.clone()]);
            }
            args.extend(prefix.iter().map(|s| s.to_string()));
            if let Some(a) = arguments.get("args") {
                let a = a
                    .as_array()
                    .ok_or_else(|| (-32602, "args must be a string array".into()))?;
                for arg in a {
                    args.push(
                        arg.as_str()
                            .ok_or_else(|| (-32602, "args must be a string array".into()))?
                            .into(),
                    );
                }
            }
            let result = match Cli::try_parse_from(args) {
                Ok(cli) if !matches!(cli.command, Commands::Mcp { .. } | Commands::Q { .. }) => {
                    execute(&cli)
                }
                Ok(_) => Err(Error::input("This command is not exposed through MCP")),
                Err(e) => {
                    let error = e.use_stderr();
                    let text = e.to_string();
                    return Ok(json!({"content":[{"type":"text","text":text}],"isError":error}));
                }
            };
            Ok(match result {
                Ok(out) => {
                    let structured =
                        json!({"result":out.value,"warnings":out.warnings,"code":out.code});
                    json!({"content":[{"type":"text","text":structured.to_string()}],"structuredContent":structured,"isError":out.code!=0})
                }
                Err(e) => {
                    json!({"content":[{"type":"text","text":serde_json::to_string(&e).unwrap()}],"isError":true})
                }
            })
        }
        _ => Err((-32601, "Method not found".into())),
    }
}
