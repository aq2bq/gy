//! The cheatsheet as a script (n-cb00, r-5fcf). The rows of
//! `crates/gy/CHEATSHEET.md` are run against a real ledger, so the form it
//! writes, the required/optional brackets, and the mark rule cannot drift from
//! the CLI. The names of commands and options are checked by `cli_docs.rs`; this
//! file checks that the drawn form is true.
use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, Output};

const CHEATSHEET: &str = "CHEATSHEET.md";
/// The text a mark names: it sits in the older decision's body.
const MARK: &str = "the old passage";
/// A mark that is not in the older decision.
const ABSENT: &str = "a passage that is not there";

/// The rows that are not run: `serve` stays up and the `remote` three speak to
/// the network. Everything else must run (d-9ae5).
const EXCLUDED: [&str; 4] = ["serve", "remote set", "remote join", "remote sync"];

/// Every command a row may name. A row whose command is not here fails, so a
/// new cheatsheet line cannot be skipped in silence.
const COMMANDS: [&str; 27] = [
    "init",
    "show",
    "list",
    "next",
    "handover",
    "publish",
    "serve",
    "cheat",
    "remote set",
    "remote join",
    "remote sync",
    "need add",
    "need close",
    "question add",
    "question close",
    "criterion add",
    "criterion satisfy",
    "req add",
    "req approve",
    "req revise",
    "req done",
    "req cancel",
    "decide",
    "link",
    "edit",
    "scope rename",
    "undo",
];

fn read(name: &str) -> String {
    let path = format!("{}/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}
fn errors(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// One cheatsheet row: its command and the tokens of the drawn form, with `[`
/// and `]` as tokens of their own so a bracket can be read.
struct Row {
    line: usize,
    section: String,
    command: String,
    words: usize,
    tokens: Vec<String>,
}

impl Row {
    /// The argument tokens (the command words taken off).
    fn args(&self) -> &[String] {
        &self.tokens[1 + self.words..]
    }
}

/// The rows of the cheatsheet: `  gy ` lines, their description cut off.
fn rows() -> Vec<Row> {
    let source = read(CHEATSHEET);
    let mut section = String::new();
    let mut rows = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        if !raw.starts_with(' ') && !raw.trim().is_empty() {
            section = raw.trim().trim_end_matches(':').to_string();
            continue;
        }
        let Some(start) = raw.find("gy ") else {
            continue;
        };
        if raw[..start].trim() != "" {
            continue;
        }
        // The description starts at the first run of two or more spaces after
        // the command; the command itself is single-spaced.
        let end = raw[start..]
            .find("  ")
            .map(|at| start + at)
            .unwrap_or(raw.len());
        let drawn = raw[..end].trim_end();
        let normalized = drawn.replace('[', " [ ").replace(']', " ] ");
        let tokens: Vec<String> = normalized.split_whitespace().map(String::from).collect();
        let (command, words) = command_path(&tokens[1..]);
        rows.push(Row {
            line: index + 1,
            section: section.clone(),
            command,
            words,
            tokens,
        });
    }
    rows
}

/// The command a row names, and how many words it takes.
fn command_path(tokens: &[String]) -> (String, usize) {
    for len in (1..=3).rev() {
        if tokens.len() >= len {
            let candidate = tokens[..len].join(" ");
            if COMMANDS.contains(&candidate.as_str()) {
                return (candidate, len);
            }
        }
    }
    panic!("unknown cheatsheet row: {}", tokens.join(" "));
}

/// The commands the cheatsheet names as creating a node, from the line that
/// lists them. The scope rule turns on this set.
fn creators() -> BTreeSet<String> {
    let source = read(CHEATSHEET);
    let line = source
        .lines()
        .find(|line| line.contains("A write that creates a node"))
        .expect("the cheatsheet names the node-creating writes");
    let inside = line
        .split('(')
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("the creating writes are named in parentheses");
    inside
        .split('/')
        .map(|part| part.trim().trim_matches('`').to_string())
        .collect()
}

/// A ledger in a tempdir, with `XDG_DATA_HOME` outside the work tree and nodes
/// made through the CLI.
struct Fixture {
    dir: tempfile::TempDir,
    data: std::path::PathBuf,
    dummy: Option<String>,
}

impl Fixture {
    fn new(scopes: &[&str]) -> Self {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let data = dir.path().join("data");
        std::fs::create_dir(&data).expect("the data directory");
        let mut toml = String::from("output = \"pub\"\n\n");
        for scope in scopes {
            toml.push_str(&format!("[scopes.{scope}]\n"));
        }
        std::fs::write(dir.path().join("gy.toml"), toml).expect("gy.toml");
        std::fs::write(dir.path().join("body.txt"), "a body\n").expect("a body file");
        Fixture {
            dir,
            data,
            dummy: None,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gy"));
        command.arg("-C").arg(self.dir.path());
        command.env("XDG_DATA_HOME", &self.data);
        command.env("GY_ACTOR", "tester");
        command
    }

    fn run(&self, args: &[String]) -> Output {
        self.command()
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("run gy {args:?}: {error}"))
    }

    fn body(&self) -> String {
        self.dir
            .path()
            .join("body.txt")
            .to_string_lossy()
            .into_owned()
    }

    /// Make one node through the CLI and answer its id.
    fn create(&mut self, args: &[&str], body: Option<&str>) -> String {
        let mut full: Vec<String> = vec!["--scope".into(), "a".into()];
        full.extend(args.iter().map(|arg| arg.to_string()));
        if let Some(body_text) = body {
            std::fs::write(self.body(), format!("{body_text}\n")).expect("the body");
            full.push("--body-file".into());
            full.push(self.body());
        }
        let output = self.run(&full);
        assert!(
            output.status.success(),
            "seeding gy {full:?} failed: {}",
            errors(&output)
        );
        parse_id(&text(&output))
    }

    /// A criterion, made once and reused as a target for needs.
    fn criterion(&mut self) -> String {
        if let Some(id) = self.dummy.clone() {
            return id;
        }
        let id = self.create(&["criterion", "add", "a criterion"], None);
        self.dummy = Some(id.clone());
        id
    }

    fn node(&mut self, kind: &str, body: Option<&str>) -> String {
        match kind {
            "criterion" => self.create(&["criterion", "add", "a criterion"], body),
            "need" => {
                let target = self.criterion();
                self.create(&["need", "add", "a need", "--targets", &target], body)
            }
            "question" => self.create(
                &[
                    "question",
                    "add",
                    "a question",
                    "--decider",
                    "someone",
                    "--options",
                    "one",
                    "--options",
                    "two",
                ],
                body,
            ),
            "decision" => self.create(&["decide", "a decision", "--scope-note", "a scope"], body),
            "requirement" => {
                let need = self.node("need", None);
                let target = self.criterion();
                self.create(
                    &[
                        "req",
                        "add",
                        "a requirement",
                        "--need",
                        &need,
                        "--targets",
                        &target,
                    ],
                    body,
                )
            }
            other => panic!("unknown node kind {other}"),
        }
    }

    fn approve(&self, id: &str) {
        let output = self.run(&[
            "--scope".into(),
            "a".into(),
            "req".into(),
            "approve".into(),
            id.to_string(),
            "--design".into(),
            "a design".into(),
            "--heard-by".into(),
            "someone".into(),
            "--evidence".into(),
            "heard".into(),
        ]);
        assert!(
            output.status.success(),
            "approve failed: {}",
            errors(&output)
        );
    }
}

fn parse_id(output: &str) -> String {
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("id: ") {
            return rest.split(" (").next().unwrap_or(rest).trim().to_string();
        }
    }
    panic!("no id in: {output}");
}

/// How a row's placeholders are filled. A placeholder that is not here fails
/// the run rather than being skipped.
#[derive(Default)]
struct Fill {
    values: BTreeMap<String, Vec<String>>,
    choices: BTreeMap<String, String>,
    repeats: BTreeMap<String, usize>,
    omit: BTreeSet<String>,
}

impl Fill {
    fn value(&mut self, placeholder: &str, value: &str) {
        self.values
            .insert(placeholder.to_string(), vec![value.to_string()]);
    }
    fn values(&mut self, placeholder: &str, values: &[&str]) {
        self.values.insert(
            placeholder.to_string(),
            values.iter().map(|v| v.to_string()).collect(),
        );
    }
    fn choice(&mut self, token: &str, value: &str) {
        self.choices.insert(token.to_string(), value.to_string());
    }
    fn repeat(&mut self, flag: &str, count: usize) {
        self.repeats.insert(flag.to_string(), count);
    }
    fn omit(&mut self, flag: &str) {
        self.omit.insert(flag.to_string());
    }
}

/// One option or positional of a row: its flag and the value tokens that follow
/// it (a positional has no flag).
struct Unit {
    flag: Option<String>,
    values: Vec<String>,
}

fn units(tokens: &[String]) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut at = 0;
    while at < tokens.len() {
        let token = &tokens[at];
        if token == "[" || token == "]" {
            at += 1;
            continue;
        }
        if token == "..." {
            // A repeat marker written outside its bracket (`[--closes <Q>]...`)
            // belongs to the option before it.
            if let Some(value) = units
                .last_mut()
                .and_then(|unit: &mut Unit| unit.values.last_mut())
            {
                value.push_str("...");
            }
            at += 1;
            continue;
        }
        if token.starts_with("--") {
            let mut values = Vec::new();
            let mut next = at + 1;
            while next < tokens.len()
                && !tokens[next].starts_with("--")
                && tokens[next] != "["
                && tokens[next] != "]"
            {
                values.push(tokens[next].clone());
                next += 1;
            }
            units.push(Unit {
                flag: Some(token.clone()),
                values,
            });
            at = next;
        } else {
            units.push(Unit {
                flag: None,
                values: vec![token.clone()],
            });
            at += 1;
        }
    }
    units
}

/// The argument vector a row draws, with its placeholders filled. `with_scope`
/// adds `--scope` when the row does not already show it.
fn build(row: &Row, fill: &Fill, with_scope: bool, scope: &str) -> Vec<String> {
    let mut argv: Vec<String> = row.command.split_whitespace().map(String::from).collect();
    let mut cursor: BTreeMap<String, usize> = BTreeMap::new();
    let has_scope = units(row.args())
        .iter()
        .any(|unit| unit.flag.as_deref() == Some("--scope"));
    for unit in units(row.args()) {
        if let Some(flag) = &unit.flag {
            if !with_scope && flag == "--scope" {
                continue;
            }
            if fill.omit.contains(flag) {
                continue;
            }
            let repeat = fill.repeats.get(flag).copied().unwrap_or(1);
            let drawn = unit
                .values
                .last()
                .is_some_and(|value| value.ends_with("..."));
            let count = if drawn { repeat } else { 1 };
            if unit.values.is_empty() {
                argv.push(flag.clone());
                continue;
            }
            for _ in 0..count {
                argv.push(flag.clone());
                for value in &unit.values {
                    argv.push(substitute(value, fill, &mut cursor));
                }
            }
        } else {
            argv.push(substitute(&unit.values[0], fill, &mut cursor));
        }
    }
    if with_scope && !has_scope {
        argv.push("--scope".into());
        argv.push(scope.to_string());
    }
    argv
}

fn substitute(token: &str, fill: &Fill, cursor: &mut BTreeMap<String, usize>) -> String {
    let mut token = token.trim_matches('"').to_string();
    if token.ends_with("...") {
        token.truncate(token.len() - 3);
    }
    if token.starts_with('<') && token.ends_with('>') {
        let values = fill
            .values
            .get(&token)
            .unwrap_or_else(|| panic!("no value for the placeholder {token}"));
        let at = cursor.entry(token.clone()).or_insert(0);
        let value = values[*at % values.len()].clone();
        *at += 1;
        return value;
    }
    if token.contains('|') {
        return fill
            .choices
            .get(&token)
            .cloned()
            .unwrap_or_else(|| token.split('|').next().unwrap().to_string());
    }
    token
}

/// How a row's placeholders are filled for its command: the nodes it needs and
/// the values it draws. A command that is not here fails.
fn fill(command: &str, f: &mut Fixture) -> Fill {
    let mut fill = Fill::default();
    fill.value("<name>", "a");
    let body = f.body();
    match command {
        "init" => fill.value("<scope>", "a"),
        "handover" | "next" | "cheat" => {}
        "show" => {
            let id = f.node("criterion", None);
            fill.value("<ID>", &id);
            fill.value("<ID|ref>", &id);
        }
        "list" => {
            let ac = f.node("criterion", None);
            fill.value("<AC>", &ac);
            fill.value("<ID>", &ac);
            fill.value("<kind>", "Need");
            fill.value("<status>", "open");
            fill.value("<text>", "a word");
            fill.value("<seq|date>", "1");
        }
        "publish" => {
            fill.value("<dir>", &f.dir.path().join("pub").to_string_lossy());
        }
        "need add" => {
            let ac = f.node("criterion", None);
            let decision = f.node("decision", None);
            fill.value("<title>", "a need");
            fill.value("<AC>", &ac);
            fill.value("<D>", &decision);
            fill.value("<path>", &body);
        }
        "need close" => {
            let id = f.node("need", None);
            fill.value("<ID>", &id);
            fill.value("<text>", "not needed");
            fill.choice("fact|external", "fact");
        }
        "question add" => {
            fill.value("<title>", "a question");
            fill.values("<text>", &["one", "two"]);
            fill.value("<path>", &body);
            fill.repeat("--options", 2);
        }
        "question close" => {
            let id = f.node("question", None);
            let decision = f.node("decision", None);
            fill.value("<ID>", &id);
            fill.value("<D>", &decision);
            fill.value("<text>", "the master said so");
            fill.choice("fact|decision|non-decision", "decision");
        }
        "criterion add" => {
            fill.value("<title>", "a criterion");
            fill.value("<path>", &body);
        }
        "criterion satisfy" => {
            let ac = f.node("criterion", None);
            let need = f.create(&["need", "add", "a need", "--targets", &ac], None);
            let requirement = f.create(
                &[
                    "req",
                    "add",
                    "a requirement",
                    "--need",
                    &need,
                    "--targets",
                    &ac,
                ],
                None,
            );
            f.approve(&requirement);
            fill.value("<AC>", &ac);
            fill.value("<text>", "measured");
        }
        "req add" => {
            let need = f.node("need", None);
            let decision = f.node("decision", None);
            let ac = f.node("criterion", None);
            fill.value("<title>", "a requirement");
            fill.value("<N>", &need);
            fill.value("<D>", &decision);
            fill.value("<AC>", &ac);
            fill.value("<URL>", "https://example.com/ref");
            fill.value("<path>", &body);
        }
        "req approve" => {
            let id = f.node("requirement", None);
            fill.value("<ID|ref>", &id);
            fill.values("<text>", &["a design", "heard"]);
        }
        "req revise" => {
            let id = f.node("requirement", None);
            f.approve(&id);
            fill.value("<ID>", &id);
            fill.values("<text>", &["a reason", "a source"]);
        }
        "req done" => {
            let id = f.node("requirement", None);
            f.approve(&id);
            fill.value("<ID>", &id);
            fill.value("<text>", "shipped");
        }
        "req cancel" => {
            let id = f.node("requirement", None);
            fill.value("<ID>", &id);
            fill.values("<text>", &["a reason", "a source"]);
        }
        "decide" => {
            let question = f.node("question", None);
            let older = f.node("decision", Some(MARK));
            fill.value("<title>", "a decision");
            fill.values("<text>", &["a scope note", MARK, "a source"]);
            fill.value("<path>", &body);
            fill.value("<Q>", &question);
            fill.value("<relation>", "narrows");
            fill.value("<D>", &older);
        }
        "link" => {
            let from = f.node("need", None);
            let to = f.node("decision", None);
            fill.value("<from>", &from);
            fill.value("<relation>", "spawned-by");
            fill.value("<to>", &to);
            // The mark rule is run in full by `marks_match_the_rule`.
            fill.omit("--mark");
            fill.omit("--remove");
        }
        "edit" => {
            let id = f.node("criterion", None);
            fill.value("<ID>", &id);
            fill.value("<text>", "tidy");
            fill.value("<title>", "a new title");
            fill.value("<path>", &body);
        }
        "scope rename" => {
            fill.value("<old>", "a");
            fill.value("<new>", "c");
        }
        "undo" => {
            f.node("criterion", None);
            fill.value("<text>", "that was wrong");
        }
        other => panic!("no fill for the command {other}"),
    }
    fill
}

fn scenario(command: &str, scopes: &[&str]) -> (Fixture, Fill) {
    let mut fixture = Fixture::new(scopes);
    // A read needs a ledger to exist; one write makes it (through the CLI).
    fixture.criterion();
    let fill = fill(command, &mut fixture);
    (fixture, fill)
}

fn run_row(row: &Row, with_scope: bool, scopes: &[&str]) -> Output {
    let (fixture, fill) = scenario(&row.command, scopes);
    fixture.run(&build(row, &fill, with_scope, scopes[0]))
}

/// (a) Every row runs as drawn, with `--scope`; a node-creating write is
/// refused without one and passes without one when only one scope exists. The
/// four excluded rows are named and nothing else is skipped.
#[test]
fn every_row_runs_as_written() {
    let creators = creators();
    let mut ran = 0;
    for row in rows() {
        if EXCLUDED.contains(&row.command.as_str()) {
            continue;
        }
        ran += 1;
        let output = run_row(&row, true, &["a", "b"]);
        assert!(
            output.status.success(),
            "line {}: gy {} (with --scope) failed: {}",
            row.line,
            row.command,
            errors(&output)
        );
        if row.section != "Writes" {
            continue;
        }
        let output = run_row(&row, false, &["a", "b"]);
        if creators.contains(&row.command) {
            assert!(
                !output.status.success() && errors(&output).contains("--scope"),
                "line {}: gy {} without --scope should ask for one: {}",
                row.line,
                row.command,
                errors(&output)
            );
        } else {
            assert!(
                output.status.success(),
                "line {}: gy {} without --scope failed: {}",
                row.line,
                row.command,
                errors(&output)
            );
        }
        let output = run_row(&row, false, &["a"]);
        assert!(
            output.status.success(),
            "line {}: gy {} without --scope in a one-scope ledger failed: {}",
            row.line,
            row.command,
            errors(&output)
        );
    }
    let expected: usize = rows()
        .iter()
        .filter(|row| !EXCLUDED.contains(&row.command.as_str()))
        .count();
    assert_eq!(ran, expected, "every non-excluded row must run");
}

/// (b) A row's brackets match the usage line's required/optional, both ways.
#[test]
fn brackets_match_the_usage() {
    for row in rows() {
        let usage = usage_required(&row.command);
        let problems = bracket_problems(&row.tokens, &usage);
        assert!(
            problems.is_empty(),
            "line {}: gy {}: {}",
            row.line,
            row.command,
            problems.join("; ")
        );
    }

    // One bracket moved must be caught: `--full` is optional in the usage, so
    // taking its brackets off makes the row claim it is required.
    let row = rows()
        .into_iter()
        .find(|row| row.command == "show" && row.tokens.contains(&"--full".to_string()))
        .expect("the show row");
    let at = row
        .tokens
        .iter()
        .position(|token| token == "--full")
        .expect("--full in the show row");
    assert_eq!(row.tokens[at - 1], "[");
    assert_eq!(row.tokens[at + 1], "]");
    let mut flipped = row.tokens.clone();
    flipped.remove(at + 1);
    flipped.remove(at - 1);
    assert!(
        !bracket_problems(&flipped, &usage_required("show")).is_empty(),
        "moving a bracket out must be caught"
    );
}

/// The options a usage line shows as required: those at the margin, before the
/// `[OPTIONS]` and the bracketed repeats.
fn usage_required(command: &str) -> BTreeSet<String> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let output = Command::new(env!("CARGO_BIN_EXE_gy"))
        .args(&args)
        .arg("--help")
        .output()
        .expect("run gy --help");
    let help = String::from_utf8_lossy(&output.stdout).into_owned();
    let line = help
        .lines()
        .find(|line| line.starts_with("Usage:"))
        .unwrap_or("");
    let mut required = BTreeSet::new();
    let mut depth = 0i32;
    for token in line.split_whitespace() {
        if depth == 0 && token.starts_with("--") {
            required.insert(token.trim_end_matches(',').to_string());
        }
        depth += token.matches('[').count() as i32;
        depth -= token.matches(']').count() as i32;
    }
    required
}

/// The options of a row outside and inside brackets.
fn bracket_options(tokens: &[String]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut required = BTreeSet::new();
    let mut optional = BTreeSet::new();
    let mut depth = 0i32;
    for token in tokens {
        if token == "[" {
            depth += 1;
            continue;
        }
        if token == "]" {
            depth -= 1;
            continue;
        }
        if token.starts_with("--") {
            if depth == 0 {
                required.insert(token.clone());
            } else {
                optional.insert(token.clone());
            }
        }
    }
    (required, optional)
}

fn bracket_problems(tokens: &[String], usage: &BTreeSet<String>) -> Vec<String> {
    let (required, optional) = bracket_options(tokens);
    let mut problems = Vec::new();
    for option in &required {
        if !usage.contains(option) {
            problems.push(format!(
                "{option} is outside [] but the usage does not require it"
            ));
        }
    }
    for option in &optional {
        if usage.contains(option) {
            problems.push(format!("{option} is inside [] but the usage requires it"));
        }
    }
    for option in usage {
        if !required.contains(option) {
            problems.push(format!("{option} is required but not outside []"));
        }
    }
    problems
}

/// The relations a mark is required with, read from the cheatsheet's own words.
fn required_from_text(source: &str) -> BTreeSet<String> {
    let mut required = BTreeSet::new();
    for line in source.lines() {
        let Some(at) = line.find("required with ") else {
            continue;
        };
        let rest = line[at + "required with ".len()..].trim_start_matches("--relate ");
        let end = rest
            .find(", optional")
            .or_else(|| rest.find(" and optional"))
            .unwrap_or(rest.len());
        for name in rest[..end].split(" and ") {
            required.insert(name.trim().to_string());
        }
    }
    required
}

/// (c) The mark rule, run in full for every relation. A mark is required
/// exactly where the cheatsheet says, a mark that resolves is accepted, and one
/// that does not is refused.
#[test]
fn marks_match_the_rule() {
    let relations = [
        "narrows",
        "widens",
        "supersedes",
        "completes",
        "targets",
        "spawned-by",
        "filed-as",
        "depends-on",
        "relies-on",
        "raised",
        "waits-on",
    ];
    let lineage = ["narrows", "widens", "supersedes", "completes"];
    let mut observed = BTreeSet::new();
    for relation in relations {
        if link_requires_mark(relation) {
            observed.insert(relation.to_string());
        }
    }
    for relation in lineage {
        if decide_requires_mark(relation) {
            observed.insert(relation.to_string());
        }
    }
    // `closes` is not formable through `link`; the rule does not reach it.
    let closes = link_is_refused("closes");
    assert!(
        closes,
        "link must refuse closes, as the cheatsheet's lines imply"
    );

    let stated = required_from_text(&read(CHEATSHEET));
    assert_eq!(
        stated, observed,
        "the cheatsheet's required marks ({stated:?}) must match the CLI's ({observed:?})"
    );

    // Swapping one relation name in the words must be caught.
    let mutated = read(CHEATSHEET).replace(
        "required with narrows and supersedes and optional",
        "required with widens and supersedes and optional",
    );
    assert_ne!(
        required_from_text(&mutated),
        observed,
        "the mark rule check would not catch a swapped relation name"
    );
}

fn flag_argv(
    command: &str,
    relation: &str,
    from: &str,
    to: &str,
    mark: Option<&str>,
) -> Vec<String> {
    let mut argv: Vec<String> = vec!["--scope".into(), "a".into()];
    argv.extend(command.split_whitespace().map(String::from));
    argv.push(from.to_string());
    argv.push(relation.to_string());
    argv.push(to.to_string());
    if let Some(mark) = mark {
        argv.push("--mark".into());
        argv.push(mark.to_string());
    }
    argv
}

/// One link relation: is a mark required? It fails if the three ways do not
/// agree with the answer.
fn link_requires_mark(relation: &str) -> bool {
    let mut fixture = Fixture::new(&["a"]);
    let (from, to) = pair(&mut fixture, relation);
    let no = fixture.run(&flag_argv("link", relation, &from, &to, None));
    if !no.status.success() {
        assert!(
            errors(&no).contains("needs a mark"),
            "{relation}: the refusal was not the mark rule: {}",
            errors(&no)
        );
        let bad = fixture.run(&flag_argv("link", relation, &from, &to, Some(ABSENT)));
        assert!(
            !bad.status.success(),
            "{relation}: a mark that does not resolve was accepted"
        );
        let good = fixture.run(&flag_argv("link", relation, &from, &to, Some(MARK)));
        assert!(
            good.status.success(),
            "{relation}: a mark that resolves was refused: {}",
            errors(&good)
        );
        return true;
    }
    // Optional: no mark is fine, a mark that does not resolve is not, and a
    // mark that resolves is fine on a fresh edge.
    let bad = fixture.run(&flag_argv("link", relation, &from, &to, Some(ABSENT)));
    assert!(
        !bad.status.success() && errors(&bad).contains("not in the older decision"),
        "{relation}: a mark that does not resolve was accepted: {}",
        errors(&bad)
    );
    let (from2, to2) = pair(&mut fixture, relation);
    let good = fixture.run(&flag_argv("link", relation, &from2, &to2, Some(MARK)));
    assert!(
        good.status.success(),
        "{relation}: a mark that resolves was refused: {}",
        errors(&good)
    );
    false
}

/// Two fresh nodes whose kinds the relation allows; the target carries MARK.
fn pair(f: &mut Fixture, relation: &str) -> (String, String) {
    match relation {
        "narrows" | "widens" | "supersedes" | "completes" => {
            (f.node("decision", None), f.node("decision", Some(MARK)))
        }
        "targets" => (f.node("need", None), f.node("criterion", Some(MARK))),
        "spawned-by" => (f.node("need", None), f.node("decision", Some(MARK))),
        "filed-as" => (f.node("need", None), f.node("requirement", Some(MARK))),
        "depends-on" => (f.node("need", None), f.node("need", Some(MARK))),
        "relies-on" => (f.node("requirement", None), f.node("decision", Some(MARK))),
        "raised" => (f.node("requirement", None), f.node("question", Some(MARK))),
        "waits-on" => (f.node("need", None), f.node("question", Some(MARK))),
        "closes" => (f.node("question", None), f.node("decision", Some(MARK))),
        other => panic!("no pair for {other}"),
    }
}

/// Whether `link` refuses `closes` whatever the mark.
fn link_is_refused(relation: &str) -> bool {
    let mut fixture = Fixture::new(&["a"]);
    let (from, to) = pair(&mut fixture, relation);
    let output = fixture.run(&flag_argv("link", relation, &from, &to, Some(MARK)));
    !output.status.success() && errors(&output).contains("cannot close")
}

/// One lineage relation through `decide --relate`: is a mark required?
fn decide_requires_mark(relation: &str) -> bool {
    let mut fixture = Fixture::new(&["a"]);
    let older = fixture.node("decision", Some(MARK));
    let decide = |mark: Option<&str>| -> Vec<String> {
        let mut argv: Vec<String> = vec!["--scope".into(), "a".into()];
        argv.extend(
            [
                "decide",
                "a decision",
                "--scope-note",
                "a scope",
                "--relate",
            ]
            .iter()
            .map(|part| part.to_string()),
        );
        argv.push(relation.to_string());
        argv.push(older.clone());
        if let Some(mark) = mark {
            argv.push("--mark".into());
            argv.push(mark.to_string());
        }
        argv
    };
    let no = fixture.run(&decide(None));
    if !no.status.success() {
        assert!(
            errors(&no).contains("needs a mark"),
            "{relation}: decide's refusal was not the mark rule: {}",
            errors(&no)
        );
        assert!(
            !fixture.run(&decide(Some(ABSENT))).status.success(),
            "{relation}: absent mark accepted"
        );
        let good = fixture.run(&decide(Some(MARK)));
        assert!(
            good.status.success(),
            "{relation}: resolved mark refused: {}",
            errors(&good)
        );
        return true;
    }
    let bad = fixture.run(&decide(Some(ABSENT)));
    assert!(
        !bad.status.success() && errors(&bad).contains("not in the older decision"),
        "{relation}: absent mark accepted: {}",
        errors(&bad)
    );
    let good = fixture.run(&decide(Some(MARK)));
    assert!(
        good.status.success(),
        "{relation}: resolved mark refused: {}",
        errors(&good)
    );
    false
}
