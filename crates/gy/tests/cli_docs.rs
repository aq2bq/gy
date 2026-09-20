//! The CLI's surface and its documents (n-ea49, r-13a4): every option the built
//! binary offers must appear in the cheatsheet and in both READMEs, and no
//! document may name an option the binary does not have. The surface is read
//! from the binary's `--help`, the thing a reader actually sees.
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

/// The options every command carries, so they are not looked for in the
/// documents: gy's global flags, and clap's own help and version.
const COMMON: [&str; 6] = ["--json", "-C", "--scope", "-h", "--help", "--version"];

const CHEATSHEET: &str = "skills/gy-loop/CHEATSHEET.md";
const README: &str = "../../README.md";
const README_JA: &str = "../../README.ja.md";

fn common() -> BTreeSet<String> {
    COMMON.iter().map(|flag| flag.to_string()).collect()
}

fn read(name: &str) -> String {
    let path = format!("{}/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn help(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_gy"))
        .args(args)
        .arg("--help")
        .output()
        .expect("run gy --help");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The lines of one clap section (`Commands:`, `Options:`), by its header.
fn section(help: &str, header: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut on = false;
    for line in help.lines() {
        let text = line.trim_end();
        if !line.starts_with(' ') && text.ends_with(':') {
            on = text == header;
            continue;
        }
        if on && !text.trim().is_empty() {
            lines.push(text.trim().to_string());
        }
    }
    lines
}

/// The options a help text offers, as `--long` (and `-x` when that is all).
fn options_of(help: &str) -> BTreeSet<String> {
    let mut options = BTreeSet::new();
    for line in section(help, "Options:") {
        for token in line.split_whitespace() {
            let flag = token.trim_end_matches(',');
            let short = flag.strip_prefix('-').is_some_and(|rest| rest.len() == 1);
            if flag.starts_with("--") || short {
                options.insert(flag.to_string());
            }
        }
    }
    options
}

/// Every command the binary offers, with its options, by its path.
fn commands(prefix: &str, text: &str, out: &mut BTreeMap<String, BTreeSet<String>>) {
    out.insert(prefix.to_string(), options_of(text));
    for line in section(text, "Commands:") {
        let name = line.split_whitespace().next().unwrap_or("");
        if name.is_empty() || name == "help" {
            continue;
        }
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix} {name}")
        };
        let args: Vec<&str> = path.split(' ').collect();
        commands(&path, &help(&args), out);
    }
}

/// A token as a row writes it: brackets, backticks, and punctuation taken off.
fn clean(token: &str) -> String {
    token
        .trim_matches(|c: char| "[]()`'\".,:;".contains(c))
        .to_string()
}

/// A document row's command and options, if it names a known command.
fn parse(span: &str, paths: &[String]) -> Option<(String, BTreeSet<String>)> {
    let tokens: Vec<String> = span.split_whitespace().map(clean).collect();
    let path = paths
        .iter()
        .filter(|path| {
            let words: Vec<&str> = path.split(' ').collect();
            words.len() <= tokens.len()
                && words
                    .iter()
                    .zip(&tokens)
                    .all(|(word, token)| token.eq_ignore_ascii_case(word))
        })
        .max_by_key(|path| path.split(' ').count())?
        .clone();
    let options = tokens
        .into_iter()
        .filter(|token| token.starts_with("--"))
        .collect();
    Some((path, options))
}

/// The rows of one document: the command, its options, and the line to fix.
fn rows(text: &str, paths: &[String]) -> BTreeMap<String, (BTreeSet<String>, usize)> {
    let mut found: BTreeMap<String, (BTreeSet<String>, usize)> = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let span = if let Some(rest) = trimmed.strip_prefix("| `") {
            rest.split('`').next().unwrap_or("")
        } else if let Some(rest) = trimmed.strip_prefix("gy ") {
            rest
        } else {
            continue;
        };
        if let Some((path, options)) = parse(span, paths) {
            let entry = found
                .entry(path)
                .or_insert_with(|| (BTreeSet::new(), index + 1));
            entry.0.extend(options);
        }
    }
    found
}

/// Both directions: an option the binary has but a document lacks, and an
/// option a document names but the binary does not have. A failure says the
/// document, the command, the option, and the line to fix.
#[test]
fn the_cli_surface_and_the_documents_agree() {
    let mut surface = BTreeMap::new();
    commands("", &help(&[]), &mut surface);
    let paths: Vec<String> = surface
        .keys()
        .filter(|path| !path.is_empty())
        .cloned()
        .collect();
    let docs = [
        ("CHEATSHEET.md", CHEATSHEET),
        ("README.md", README),
        ("README.ja.md", README_JA),
    ];
    let mut failures = Vec::new();
    for (name, path) in docs {
        let found = rows(&read(path), &paths);
        for (path, options) in &surface {
            if path.is_empty() {
                continue;
            }
            let (shown, line) = found.get(path).cloned().unwrap_or((BTreeSet::new(), 0));
            for option in options
                .difference(&common())
                .filter(|option| !shown.contains(*option))
            {
                let where_ = if line == 0 {
                    "no row".to_string()
                } else {
                    format!(":{line}")
                };
                failures.push(format!("{name}{where_} {path} is missing {option}"));
            }
        }
        for (path, (options, line)) in &found {
            let known = surface.get(path).cloned().unwrap_or_default();
            for option in options
                .difference(&common())
                .filter(|option| !known.contains(*option))
            {
                failures.push(format!(
                    "{name}:{line} {path} names {option}, which the binary does not offer"
                ));
            }
        }
        println!("{name}: {} commands matched", found.len());
    }
    // Every leaf command (one with no subcommands) must appear as a row in all
    // three documents (n-29b3). A parent like `need` is a name, not an
    // operation, so it is not required.
    for (name, path) in docs {
        let found = rows(&read(path), &paths);
        for command in &paths {
            let parent = paths
                .iter()
                .any(|other| other.starts_with(&format!("{command} ")));
            if !parent && !found.contains_key(command) {
                failures.push(format!("{name} has no row for {command}"));
            }
        }
    }
    println!("the surface, {} commands:", paths.len());
    for (path, options) in &surface {
        if !path.is_empty() {
            let list: Vec<&str> = options.iter().map(String::as_str).collect();
            println!("  {path}: {}", list.join(" "));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
