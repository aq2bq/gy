use crate::*;
use serde_json::{Value, json};
use std::{fs, path::Path};

#[derive(Default)]
pub struct QuestionOptions<'a> {
    pub decider: &'a str,
    pub options: &'a [String],
    pub bundle: Option<&'a str>,
    pub rationale: Option<&'a str>,
    pub force: bool,
}
#[derive(Default)]
pub struct AdvanceOptions<'a> {
    pub to: &'a str,
    pub evidence: &'a str,
    pub reported_base: Option<&'a str>,
    pub reported_files: Option<u64>,
    pub data_migration: Option<bool>,
    pub production_only: Option<bool>,
    pub production_done: Option<bool>,
    pub cleanup_done: Option<bool>,
}
impl Store {
    pub fn add_need(
        &mut self,
        scope: &str,
        title: &str,
        targets: &[String],
        spawned: Option<&str>,
    ) -> Result<String> {
        if targets.is_empty() {
            return Err(Error::input(
                "--targets is required. A need without an acceptance criterion disappears from the progress overview. Example: gy need add \"...\" --targets AC-3",
            ));
        }
        for id in targets {
            self.typed(id, "criterion")?;
        }
        if let Some(id) = spawned {
            self.typed(id, "decision")?;
        }
        let id = self.create("need", title, scope, None)?;
        self.nodes.get_mut(&id).unwrap().put("status", "unfiled");
        for target in targets {
            self.link(&id, "targets", target, None)?;
        }
        if let Some(d) = spawned {
            self.link(&id, "spawned-by", d, None)?;
        }
        Ok(id)
    }
    pub fn file_need(&mut self, id: &str, issue: u64) -> Result<String> {
        let need = self.typed(id, "need")?;
        let req = format!("#{issue}");
        let parent = self.config.scopes.get(need.scope()).and_then(|s| s.parent_issue).or(self.config.parent_issue).ok_or_else(|| Error::input("Set parent_issue in gy.toml. gy checks the record to confirm that the requirement is a sub-issue of the parent"))?;
        let requirement = self.typed(&req, "requirement")?;
        if requirement
            .attrs
            .get("parent_issue")
            .and_then(Value::as_u64)
            != Some(parent)
        {
            return Err(Error::input(format!(
                "The parent_issue of {req} does not match the configured value {parent}. Verify the sub-issue relationship and record it in frontmatter"
            )));
        }
        if !need.refs("filed-as").is_empty() && !need.refs("filed-as").contains(&req) {
            return Err(Error::input(format!(
                "{id} is already filed as a different requirement"
            )));
        }
        if need.refs("filed-as").contains(&req) {
            // Repeating an already recorded filing must not rewind a requirement.
            return Ok(req);
        }
        if !matches!(
            base_state(requirement.get("status")),
            Some("unfiled" | "defining" | "awaiting-design")
        ) {
            return Err(Error::input(format!(
                "{req} is already {}. Filing cannot move its state back to awaiting-design",
                requirement.get("status")
            )));
        }
        self.link(id, "filed-as", &req, None)?;
        self.nodes
            .get_mut(id)
            .unwrap()
            .put("status", "awaiting-design");
        self.nodes
            .get_mut(&req)
            .unwrap()
            .put("status", "awaiting-design");
        Ok(req)
    }
    pub fn add_question(
        &mut self,
        scope: &str,
        title: &str,
        opts: QuestionOptions<'_>,
    ) -> Result<(String, Vec<String>)> {
        if opts.decider.trim().is_empty() {
            return Err(Error::input(
                "--decider is required. Whose agreement is needed affects how long it takes to close a question",
            ));
        }
        if opts
            .options
            .iter()
            .filter(|s| !s.trim().is_empty())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            < 2
        {
            return Err(Error::input(
                "--options requires at least two distinct, viable options. Something uniquely determined by facts is not a question to decide",
            ));
        }
        if opts.bundle.is_some() && opts.rationale.is_none_or(|s| s.trim().is_empty()) {
            return Err(Error::input(
                "--bundle-rationale is required. Explain why the same intervention closes the bundled questions; a shared cause is insufficient",
            ));
        }
        let hits = self.find(Some(title), &[], None)?;
        let warnings: Vec<String> = hits
            .iter()
            .map(|h| format!("Existing record: {} [{}] {}", h.id, h.section, h.excerpt))
            .collect();
        if !warnings.is_empty() && !opts.force {
            return Err(Error::input(format!(
                "{}\nCheck whether this question is already answered. Review the records, then use --force if a new question is needed",
                warnings.join("\n")
            )));
        }
        let id = self.create("question", title, scope, None)?;
        let n = self.nodes.get_mut(&id).unwrap();
        n.put("status", "open");
        n.put("decider", opts.decider);
        n.put("options", opts.options);
        if let Some(b) = opts.bundle {
            n.put("bundle", b);
            n.put("bundle-rationale", opts.rationale.unwrap());
        }
        Ok((id, warnings))
    }
    pub fn close_question(
        &mut self,
        id: &str,
        by: &str,
        decision: Option<&str>,
        note: Option<&str>,
    ) -> Result<Vec<String>> {
        self.typed(id, "question")?;
        if !["fact", "decision", "non-decision"].contains(&by) {
            return Err(Error::input(
                "--by must be fact / decision / non-decision. Both decision and non-decision require a recorded decision",
            ));
        }
        if by == "fact" {
            if note.is_none_or(|s| s.trim().is_empty()) {
                return Err(Error::input(
                    "--by fact requires --note. Record why facts resolved the question without a choice",
                ));
            }
            if decision.is_some() {
                return Err(Error::input(
                    "--decision cannot be used with fact. Use --by decision to close with a decision",
                ));
            }
        } else {
            let d = decision.filter(|s| !s.is_empty()).ok_or_else(|| Error::input("Closing by decision / non-decision requires --decision. First record a decision with gy decide \"...\" --scope-note \"applicability conditions\""))?;
            self.typed(d, "decision")?;
        }
        if self.node(id)?.closed() {
            return Err(Error::input(format!(
                "{id} is already closed. Its previous closure cannot be overwritten"
            )));
        }
        if let Some(d) = decision {
            self.link(id, "closes", d, None)?;
        }
        let n = self.nodes.get_mut(id).unwrap();
        n.put("status", "closed");
        n.put("closed_by", by);
        n.put("closed_at", chrono::Utc::now().to_rfc3339());
        if let Some(note) = note {
            n.put("closure_note", note);
        }
        Ok(self
            .waiting_references(id)
            .into_iter()
            .map(|r| {
                format!(
                    "{r} references closed question {id} as unresolved. Review the referring node"
                )
            })
            .collect())
    }
    pub fn decide(
        &mut self,
        scope: &str,
        title: &str,
        scope_note: &str,
        closes: &[String],
    ) -> Result<(String, Vec<String>)> {
        if scope_note.trim().is_empty() {
            return Err(Error::input(
                "--scope-note (decision applicability conditions) is required. Without it, later users may apply the decision too broadly. Example: gy decide \"...\" --scope-note \"Applies only to delivery paths using the priority queue; excludes batch replays\"",
            ));
        }
        for q in closes {
            if self.typed(q, "question")?.closed() {
                return Err(Error::input(format!("{q} is already closed")));
            }
        }
        let mut warnings: Vec<String> = self
            .nodes
            .values()
            .filter(|n| n.kind() == "question" && !n.closed() && !closes.contains(&n.id().into()))
            .map(|n| {
                format!(
                    "Open question {}: {} — check whether this decision answers it",
                    n.id(),
                    n.get("title")
                )
            })
            .collect();
        let id = self.create("decision", title, scope, None)?;
        self.nodes
            .get_mut(&id)
            .unwrap()
            .put("decision_scope", scope_note);
        for q in closes {
            warnings.extend(self.close_question(q, "decision", Some(&id), None)?);
        }
        Ok((id, warnings))
    }
    pub fn set_attributes(
        &mut self,
        id: &str,
        attrs: &serde_json::Map<String, Value>,
        body: Option<String>,
    ) -> Result<()> {
        self.node(id)?;
        if attrs
            .get("title")
            .is_some_and(|v| v.as_str().is_none_or(|s| s.trim().is_empty()))
        {
            return Err(Error::input("title must be a nonempty, descriptive string"));
        }
        for key in attrs.keys() {
            if [
                "capture",
                "transitions",
                "closed_at",
                "closed_by",
                "compressed",
                "compressed_from",
            ]
            .contains(&key.as_str())
            {
                return Err(Error::input(format!(
                    "{key} is recorded by its dedicated command"
                )));
            }
            if key == "status" && ["question", "requirement"].contains(&self.node(id)?.kind()) {
                return Err(Error::input(
                    "Use question close to close questions and req advance to change requirement states with evidence",
                ));
            }
            if ["id", "type", "scope", "created"].contains(&key.as_str()) {
                return Err(Error::input(format!(
                    "{key} cannot be changed. IDs and identity attributes are immutable"
                )));
            }
            if EDGES.iter().any(|e| key == e.0 || key == e.3) {
                return Err(Error::input(format!(
                    "Use gy link to update {key} on both sides"
                )));
            }
        }
        let n = self.nodes.get_mut(id).unwrap();
        for (key, value) in attrs {
            n.attrs.insert(key.clone(), value.clone());
        }
        if let Some(body) = body {
            n.body = body;
        }
        if n.attrs.contains_key("compressed") {
            n.body = compressed_body(n)?;
        }
        Ok(())
    }
    pub fn advance(&mut self, id: &str, opts: AdvanceOptions<'_>) -> Result<()> {
        let n = self.typed(id, "requirement")?;
        if opts.evidence.trim().is_empty() {
            return Err(Error::input(
                "--evidence is required. Provide verification results or a record location so every state transition retains its evidence",
            ));
        }
        let state = base_state(opts.to).ok_or_else(|| {
            Error::input(format!(
                "--to must be one of these 11 states (parenthesized context is allowed): {}",
                STATES.join(" / ")
            ))
        })?;
        if state == "awaiting-merge" {
            let url = n.get("pr_url");
            if !valid_url(url) || !url.contains("/pull/") {
                return Err(Error::input(
                    "To enter awaiting-merge, record the PR URL in frontmatter pr_url. gy does not query GitHub",
                ));
            }
            if n.get("pr_base").is_empty()
                || opts.reported_base != Some(n.get("pr_base"))
                || opts.reported_files.is_none()
                || opts.reported_files != n.attrs.get("pr_files").and_then(Value::as_u64)
            {
                return Err(Error::input(
                    "Supply --reported-base / --reported-files matching frontmatter pr_base / pr_files",
                ));
            }
        }
        if state == "complete" {
            let issues = self.completion_issues(n);
            if !issues.is_empty() {
                return Err(Error::input(issues.join("\n")));
            }
        }
        if ["awaiting-production", "awaiting-cleanup", "complete"].contains(&state) {
            let migration = opts.data_migration.ok_or_else(|| {
                Error::input(
                    "Use --data-migration true|false to report whether data migration is required",
                )
            })?;
            let production = opts.production_only.ok_or_else(|| Error::input("Use --production-only true|false to report whether the change can only be verified in production. This is separate from whether migration is required"))?;
            let production_done = opts
                .production_done
                .or_else(|| n.attrs.get("production_done").and_then(Value::as_bool))
                .unwrap_or(false);
            let cleanup_done = opts
                .cleanup_done
                .or_else(|| n.attrs.get("cleanup_done").and_then(Value::as_bool))
                .unwrap_or(false);
            let expected = if (migration || production) && !production_done {
                "awaiting-production"
            } else if !cleanup_done || remaining(n) != Some(0) {
                "awaiting-cleanup"
            } else {
                "complete"
            };
            if state != expected {
                return Err(Error::input(format!(
                    "The reported migration, production verification, and remaining work imply state {expected}. Check --production-done / --cleanup-done and remaining_work"
                )));
            }
        }
        if state == "complete" && remaining(n) != Some(0) {
            return Err(Error::input(
                "Completion requires remaining_work = 0. List remaining work and verify it is complete",
            ));
        }
        let previous = n.get("status").to_owned();
        let n = self.nodes.get_mut(id).unwrap();
        let mut history = n
            .attrs
            .get("transitions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        history.push(json!({"from":previous,"to":opts.to,"evidence":opts.evidence,"at":chrono::Utc::now().to_rfc3339(),"reported_base":opts.reported_base,"reported_files":opts.reported_files,"data_migration":opts.data_migration,"production_only":opts.production_only,"production_done":opts.production_done,"cleanup_done":opts.cleanup_done}));
        n.put("transitions", history);
        n.put("status", opts.to);
        n.put("evidence", opts.evidence);
        for (key, value) in [
            ("data_migration", opts.data_migration),
            ("production_only", opts.production_only),
            ("production_done", opts.production_done),
            ("cleanup_done", opts.cleanup_done),
        ] {
            if let Some(value) = value {
                n.put(key, value);
            }
        }
        Ok(())
    }
    /// Prepare an archive only after every recorded downstream constraint has
    /// a decision node. Uploading this text remains the caller's responsibility.
    pub fn compression_preview(&self, id: &str) -> Result<String> {
        let n = self.typed(id, "requirement")?;
        if n.attrs.contains_key("compressed") {
            return Err(Error::input(format!(
                "{id} is already compressed. See compressed_from for the original full text"
            )));
        }
        if base_state(n.get("status")) != Some("complete") || remaining(n) != Some(0) {
            return Err(Error::input(
                "Only completed requirements with zero remaining work can be compressed. First record completion evidence with req advance",
            ));
        }
        let constraints = n.attrs.get("constraints").and_then(Value::as_array).ok_or_else(|| Error::input("Record each downstream constraint in constraints. If review finds no constraints, record an empty array"))?;
        let mut missing = vec![];
        for (i, constraint) in constraints.iter().enumerate() {
            let description = constraint
                .get("text")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty());
            let decision = constraint.get("decision").and_then(Value::as_str);
            match (description, decision) {
                (Some(description), Some(id))
                    if self.nodes.get(id).is_some_and(|d| {
                        d.kind() == "decision" && !d.get("decision_scope").trim().is_empty()
                    }) =>
                {
                    if !n.refs("relies-on").contains(&id.to_string()) {
                        missing.push(format!("{description}: record the decision dependency with gy link {} relies-on {id}",n.id()));
                    }
                }
                (description, _) => missing.push(format!(
                    "Constraint {}: {}",
                    i + 1,
                    description.unwrap_or("text is missing")
                )),
            }
        }
        if !missing.is_empty() {
            return Err(Error::input(format!(
                "Cannot compress because downstream constraints are not recorded as decisions. Use gy decide to record decisions with applicability conditions.\n{}",
                missing.join("\n")
            )));
        }
        if n.attrs.get("constraints_reviewed").and_then(Value::as_bool) != Some(true) {
            return Err(Error::input(
                "constraints_reviewed=true is required to record review of each constraint. The caller must check whether any constraints were omitted from the body",
            ));
        }
        let issues = self.compression_field_issues(n);
        if !issues.is_empty() {
            return Err(Error::input(issues.join("\n")));
        }
        Ok(fs::read_to_string(&n.path)?)
    }
    pub fn import_adr(&mut self, directory: &Path, scope: &str) -> Result<Vec<String>> {
        let mut imported = vec![];
        let mut paths: Vec<_> =
            fs::read_dir(directory)?.collect::<std::result::Result<Vec<_>, _>>()?;
        paths.sort_by_key(|e| e.path());
        for entry in paths {
            let path = entry.path();
            if path.extension().is_none_or(|x| x != "md") {
                continue;
            }
            let raw = fs::read_to_string(&path)?;
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let first = name
                .strip_prefix("D-")
                .unwrap_or(name)
                .split('-')
                .next()
                .unwrap_or("");
            let mut node;
            if raw.starts_with("---\n") {
                let rest = raw.strip_prefix("---\n").unwrap();
                let (yaml, body) = rest.split_once("\n---\n").ok_or_else(|| {
                    Error::input(format!(
                        "{}: missing closing frontmatter delimiter",
                        path.display()
                    ))
                })?;
                let mut attrs: serde_json::Map<String, Value> =
                    serde_yaml::from_str(yaml).map_err(|e| Error::input(e.to_string()))?;
                let original = attrs.get("id").map(text).unwrap_or_else(|| first.into());
                let id = if original.starts_with("D-") {
                    original
                } else {
                    format!(
                        "D-{}",
                        original.parse::<u64>().map_err(|_| Error::input(format!(
                            "{}: cannot determine the decision ID",
                            path.display()
                        )))?
                    )
                };
                let title = attrs
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or(name)
                    .to_string();
                node = Node::new(&id, "decision", &title, scope);
                attrs.remove("id");
                attrs.remove("type");
                attrs.remove("scope");
                node.attrs.extend(attrs);
                node.body = body.into();
            } else {
                let number: u64 = match first.parse() {
                    Ok(n) if n > 0 => n,
                    _ => continue,
                };
                let id = if name.starts_with("D-") {
                    format!("D-{first}")
                } else {
                    format!("D-{number}")
                };
                let title = raw
                    .lines()
                    .find_map(|l| l.strip_prefix("# "))
                    .unwrap_or(name);
                node = Node::new(&id, "decision", title, scope);
                node.body = raw;
            }
            if !valid_id(node.id(), "decision") {
                return Err(Error::input(format!("{}: invalid ID", path.display())));
            }
            if self.nodes.contains_key(node.id()) {
                return Err(Error::input(format!(
                    "Import aborted because {} already exists. IDs are not changed",
                    node.id()
                )));
            }
            node.path = self
                .root
                .join(scope)
                .join("decisions")
                .join(format!("{}.md", node.id()));
            imported.push(node.id().to_owned());
            self.nodes.insert(node.id().to_owned(), node);
        }
        Ok(imported)
    }
}
pub fn valid_url(s: &str) -> bool {
    s.split_once("://").is_some_and(|(scheme, rest)| {
        ["https", "http"].contains(&scheme)
            && rest
                .split('/')
                .next()
                .is_some_and(|host| !host.is_empty() && !host.contains('@'))
    }) && !s.chars().any(char::is_whitespace)
}
