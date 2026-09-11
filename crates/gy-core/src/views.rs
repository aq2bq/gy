use crate::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    process::Command,
};

#[derive(Debug, Serialize)]
pub struct Hit {
    pub id: String,
    pub scope: String,
    pub section: String,
    pub excerpt: String,
}
fn attribute<'a>(n: &'a Node, path: &str) -> Option<&'a Value> {
    if let Some(v) = n.attrs.get(path) {
        return Some(v);
    }
    let mut parts = path.split('.');
    let mut value = n.attrs.get(parts.next()?)?;
    for part in parts {
        value = value.get(part)?;
    }
    Some(value)
}
fn matches_filter(n: &Node, expression: &str) -> Result<bool> {
    let (key, op, expected) = [">=", "<=", "!=", "=", ">", "<", "~"].iter().find_map(|op| expression.split_once(op).map(|(k,v)| (k.trim(), *op, v.trim()))).ok_or_else(|| Error::input(format!("Invalid attribute filter {expression}. Example: --where decider=master / --where 'created>=2026-09-01'")))?;
    if key.is_empty() {
        return Err(Error::input("The attribute name in --where is empty"));
    }
    let Some(value) = attribute(n, key) else {
        return Ok(false);
    };
    let actual = text(value);
    let ordering = match (value.as_f64(), expected.parse::<f64>()) {
        (Some(a), Ok(b)) => a.partial_cmp(&b),
        _ => Some(actual.as_str().cmp(expected)),
    };
    Ok(match op {
        "=" => {
            actual == expected
                || value
                    .as_array()
                    .is_some_and(|a| a.iter().any(|v| text(v) == expected))
        }
        "!=" => actual != expected,
        "~" => actual.contains(expected),
        ">=" => ordering.is_some_and(|o| !o.is_lt()),
        "<=" => ordering.is_some_and(|o| !o.is_gt()),
        ">" => ordering.is_some_and(|o| o.is_gt()),
        "<" => ordering.is_some_and(|o| o.is_lt()),
        _ => false,
    })
}
impl Store {
    pub fn find(
        &self,
        keyword: Option<&str>,
        filters: &[String],
        scope: Option<&str>,
    ) -> Result<Vec<Hit>> {
        let mut result = vec![];
        // Validate syntax even when the ledger is empty.
        for f in filters {
            matches_filter(&Node::new("N-1", "need", "", "s"), f)?;
        }
        let query = keyword.unwrap_or("").to_lowercase();
        for n in self
            .nodes
            .values()
            .filter(|n| scope.is_none_or(|s| n.scope() == s))
        {
            let mut matches = true;
            for f in filters {
                if !matches_filter(n, f)? {
                    matches = false;
                    break;
                }
            }
            if !matches {
                continue;
            }
            if query.is_empty() {
                result.push(Hit {
                    id: n.id().into(),
                    scope: n.scope().into(),
                    section: "title".into(),
                    excerpt: n.get("title").into(),
                });
                continue;
            }
            for (key, value) in &n.attrs {
                let value = text(value);
                if value.to_lowercase().contains(&query) || key.to_lowercase().contains(&query) {
                    result.push(Hit {
                        id: n.id().into(),
                        scope: n.scope().into(),
                        section: format!("frontmatter.{key}"),
                        excerpt: value,
                    });
                }
            }
            let mut section = "Body";
            for line in n.body.lines() {
                if line.starts_with('#') {
                    section = line.trim_start_matches('#').trim();
                }
                if line.to_lowercase().contains(&query) {
                    result.push(Hit {
                        id: n.id().into(),
                        scope: n.scope().into(),
                        section: section.into(),
                        excerpt: line.into(),
                    });
                }
            }
        }
        Ok(result)
    }
    pub fn display_node(&self, n: &Node) -> Result<String> {
        let mut copy = n.clone();
        if n.attrs.contains_key("compressed") {
            copy.body = compressed_body(n)?;
        }
        for (key, label) in [
            ("narrowed-by", "Applicability narrowed"),
            ("superseded-by", "Superseded"),
        ] {
            for source in n.refs(key) {
                if let Some(anchor) = edge_mark(n, key, &source) {
                    let annotation = format!(" ⟦{label}: {source}⟧");
                    if copy.body.contains(&anchor) {
                        copy.body = copy.body.replace(&anchor, &format!("{anchor}{annotation}"));
                    } else {
                        copy.body.push_str(&format!("\n> {label}: {source} — Affected passage: {anchor} (location in body could not be found)\n"));
                    }
                } else {
                    copy.body.push_str(&format!(
                        "\n> {label}: {source} (no mark identifies the affected passage)\n"
                    ));
                }
            }
        }
        if n.closed() {
            let reason = if n.get("closed_by") == "fact" {
                format!("Resolved by fact: {}", n.get("closure_note"))
            } else {
                format!("→ Decided by {}", n.refs("closes").join(", "))
            };
            copy.body = format!("\n> {reason}\n{}", copy.body);
        }
        copy.markdown()
    }
    pub fn neighbors(&self, id: &str) -> Result<Vec<Value>> {
        let n = self.node(id)?;
        let mut out = vec![];
        for (label, _, _, reverse) in EDGES {
            let keys = if label == reverse {
                vec![*label]
            } else {
                vec![*label, *reverse]
            };
            for key in keys {
                for id in n.refs(key) {
                    out.push(json!({"relation":key,"id":id,"title":self.nodes.get(&id).map(|n| n.get("title")),"mark":edge_mark(n,key,&id)}));
                }
            }
        }
        Ok(out)
    }
    pub fn dot(&self, scope: Option<&str>, start: Option<&str>) -> Result<String> {
        let mut selected: BTreeSet<String> = self
            .nodes
            .values()
            .filter(|n| scope.is_none_or(|s| n.scope() == s))
            .map(|n| n.id().into())
            .collect();
        if let Some(start) = start {
            self.node(start)?;
            let mut reached = BTreeSet::new();
            let mut queue = vec![start.to_string()];
            while let Some(id) = queue.pop() {
                if !selected.contains(&id) || !reached.insert(id.clone()) {
                    continue;
                }
                for v in self.neighbors(&id)? {
                    if let Some(id) = v["id"].as_str() {
                        queue.push(id.into());
                    }
                }
            }
            selected = reached;
        }
        let quote = |s: &str| serde_json::to_string(s).unwrap();
        let mut out = "digraph gy {\n  rankdir=LR;\n".to_string();
        for id in &selected {
            let n = &self.nodes[id];
            out.push_str(&format!(
                "  {} [label={}];\n",
                quote(id),
                quote(&format!("{}: {}", id, n.get("title")))
            ));
        }
        for id in &selected {
            let n = &self.nodes[id];
            for (label, source, _, _) in EDGES {
                if !kind_matches(n.kind(), source) {
                    continue;
                }
                for target in n.refs(label) {
                    if selected.contains(&target) {
                        out.push_str(&format!(
                            "  {} -> {} [label={}];\n",
                            quote(id),
                            quote(&target),
                            quote(label)
                        ));
                    }
                }
            }
        }
        out.push_str("}\n");
        Ok(out)
    }
    pub fn render(&self, scope: Option<&str>, format: &str) -> Result<Vec<PathBuf>> {
        if !["markdown", "dot"].contains(&format) {
            return Err(Error::input("--format must be markdown / dot"));
        }
        let mut files = BTreeMap::new();
        if format == "dot" {
            files.insert(PathBuf::from("graph.dot"), self.dot(scope, None)?);
        } else {
            let mut scopes: BTreeSet<_> =
                self.nodes.values().map(|n| n.scope().to_owned()).collect();
            scopes.extend(self.config.scopes.keys().cloned());
            let scope_count = scopes.len();
            for s in scopes.into_iter().filter(|s| scope.is_none_or(|x| x == s)) {
                let output = self.config.render.output.replace("{scope}", &s);
                if scope.is_none()
                    && !self.config.render.output.contains("{scope}")
                    && scope_count > 1
                {
                    return Err(Error::input(
                        "render.output must include {scope} when rendering multiple scopes",
                    ));
                }
                let path = PathBuf::from(output);
                let parent = path.parent().unwrap_or(std::path::Path::new(""));
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("README");
                let nodes: Vec<_> = self.nodes.values().filter(|n| n.scope() == s).collect();
                let chunks: Vec<_> = nodes.chunks(self.config.render.split_threshold).collect();
                if chunks.len() <= 1 {
                    files.insert(path, self.render_page(&s, &nodes)?);
                } else {
                    let mut index = format!("# {s}\n\n{} nodes\n\n", nodes.len());
                    for (i, chunk) in chunks.iter().enumerate() {
                        let name = format!("{stem}-{}.md", i + 1);
                        index.push_str(&format!(
                            "- [{}–{}]({name})\n",
                            i * self.config.render.split_threshold + 1,
                            i * self.config.render.split_threshold + chunk.len()
                        ));
                        files.insert(parent.join(name), self.render_page(&s, chunk)?);
                    }
                    files.insert(path, index);
                }
            }
        }
        for path in files.keys() {
            if path
                .components()
                .nth(1)
                .is_some_and(|c| KINDS.iter().any(|k| c.as_os_str() == k.2))
            {
                return Err(Error::input(
                    "render output cannot be placed in source node directories",
                ));
            }
            if self.nodes.values().any(|n| n.path == self.root.join(path))
                || path
                    .file_name()
                    .is_some_and(|n| n == "gy.toml" || n.to_string_lossy().starts_with(".gy"))
            {
                return Err(Error::input(
                    "render output overlaps a source or management file",
                ));
            }
        }
        let paths = files.keys().map(|p| self.root.join(p)).collect();
        self.write_files(files)?;
        Ok(paths)
    }
    fn render_page(&self, scope: &str, nodes: &[&Node]) -> Result<String> {
        fn escape(s: &str) -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('|', "&#124;")
                .replace('\n', "<br>")
        }
        let mut out =
            format!("# {scope}\n\n| ID | Type | Title | Status |\n| --- | --- | --- | --- |\n");
        for n in nodes {
            let content = if n.closed() {
                if n.get("closed_by") == "fact" {
                    format!("Resolved by fact: {}", n.get("closure_note"))
                } else {
                    format!("→ Decided by {}", n.refs("closes").join(", "))
                }
            } else {
                n.get("title").into()
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                escape(n.id()),
                escape(n.kind()),
                escape(&content),
                escape(n.get("status"))
            ));
        }
        for n in nodes {
            out.push_str(&format!("\n## {} — {}\n\n", n.id(), escape(n.get("title"))));
            let md = self.display_node(n)?;
            let (yaml, body) = md
                .strip_prefix("---\n")
                .unwrap()
                .split_once("\n---\n")
                .unwrap();
            let longest_fence = yaml
                .lines()
                .map(|line| line.chars().take_while(|c| *c == '`').count())
                .max()
                .unwrap_or(0);
            let fence = "`".repeat(3.max(longest_fence + 1));
            out.push_str(&format!(
                "<details><summary>Attributes</summary>\n\n{fence}yaml\n{yaml}\n{fence}\n\n</details>\n\n"
            ));
            out.push_str(body);
            out.push('\n');
        }
        Ok(out)
    }
    pub fn next(&self, scope: Option<&str>) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| {
                n.kind() == "need"
                    && scope.is_none_or(|s| s == n.scope())
                    && !self.need_done(n)
                    && n.refs("depends-on")
                        .iter()
                        .all(|id| self.nodes.get(id).is_some_and(|d| self.need_done(d)))
                    && !self.blocked(n, &mut BTreeSet::new())
            })
            .collect()
    }
    fn need_done(&self, n: &Node) -> bool {
        base_state(n.get("status")) == Some("complete")
            || (!n.refs("filed-as").is_empty()
                && n.refs("filed-as").iter().all(|id| {
                    self.nodes
                        .get(id)
                        .is_some_and(|r| base_state(r.get("status")) == Some("complete"))
                }))
    }
    fn blocked(&self, n: &Node, seen: &mut BTreeSet<String>) -> bool {
        if !seen.insert(n.id().into()) {
            return false;
        }
        for key in [
            "waiting-on",
            "unresolved",
            "filed-as",
            "raised",
            "spawned-by",
            "relies-on",
            "closes",
            "depends-on",
        ] {
            for id in n.refs(key) {
                match self.nodes.get(&id) {
                    None => return true,
                    Some(target) if target.kind() == "question" => {
                        if !target.closed() {
                            return true;
                        }
                    }
                    Some(target) => {
                        if self.blocked(target, seen) {
                            return true;
                        }
                    }
                }
            }
        }
        self.nodes.values().any(|q| {
            q.kind() == "question" && !q.closed() && q.refs("belongs-to").contains(&n.id().into())
        })
    }
    pub fn handover(&self, scope: Option<&str>) -> Value {
        let diagnostics = self.lint(scope);
        let active: Vec<_> = self
            .nodes
            .values()
            .filter(|n| {
                n.kind() == "requirement"
                    && base_state(n.get("status")) != Some("complete")
                    && scope.is_none_or(|s| s == n.scope())
            })
            .collect();
        let missing: Vec<_> = active.iter().filter(|n| n.get("next_evidence").trim().is_empty() || n.get("responsible").trim().is_empty()).map(|n| json!({"id":n.id(),"next_evidence":n.get("next_evidence"),"responsible":n.get("responsible")})).collect();
        let dangling: Vec<_> = self
            .nodes
            .values()
            .filter(|n| scope.is_none_or(|s| n.scope() == s))
            .flat_map(|n| {
                n.attrs
                    .iter()
                    .filter(|(k, _)| k.as_str() != "id")
                    .flat_map(|(_, v)| attribute_ids(v))
                    .filter(|id| !self.nodes.contains_key(id))
                    .map(|id| json!({"source":n.id(),"target":id}))
                    .collect::<Vec<_>>()
            })
            .collect();
        json!({"lint":diagnostics,"active_requirements":active.len(),"with_next_evidence_and_responsible":active.len()-missing.len(),"missing":missing,"dangling":dangling,"note":INTEGRITY_NOTE})
    }
    pub fn stats(&self, scope: Option<&str>, days: u32) -> Result<Value> {
        if days == 0 {
            return Err(Error::input("--days must be at least 1"));
        }
        let criteria: Vec<_> = self.nodes.values().filter(|n| n.kind() == "criterion" && scope.is_none_or(|s| s == n.scope())).map(|n| json!({"id":n.id(),"title":n.get("title"),"satisfied":n.attrs.get("satisfied").and_then(Value::as_bool).unwrap_or(false),"satisfied_at":n.attrs.get("satisfied_at")})).collect();
        let satisfied = criteria.iter().filter(|n| n["satisfied"] == true).count();
        let history = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args([
                "log",
                "--all",
                "--diff-filter=A",
                "--format=GY-COMMIT:%ct",
                "--name-only",
                "--",
                ".",
            ])
            .output();
        let now = chrono::Utc::now().timestamp();
        let window = i64::from(days) * 86400;
        let rate = match history {
            Ok(output) if output.status.success() => {
                let mut timestamp = 0i64;
                let mut births = BTreeMap::<String, i64>::new();
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if let Some(t) = line.strip_prefix("GY-COMMIT:") {
                        timestamp = t.parse().unwrap_or(0);
                        continue;
                    }
                    let p = std::path::Path::new(line);
                    if p.parent()
                        .and_then(|p| p.file_name())
                        .is_none_or(|s| s != "questions")
                        || p.extension().is_none_or(|s| s != "md")
                    {
                        continue;
                    }
                    if scope.is_some_and(|s| {
                        p.parent()
                            .and_then(|p| p.parent())
                            .and_then(|p| p.file_name())
                            .is_none_or(|v| v != s)
                    }) {
                        continue;
                    }
                    if let Some(id) = p.file_stem().and_then(|s| s.to_str()) {
                        births
                            .entry(id.into())
                            .and_modify(|old| *old = (*old).min(timestamp))
                            .or_insert(timestamp);
                    }
                }
                let current = births
                    .values()
                    .filter(|t| **t >= now - window && **t <= now)
                    .count();
                let previous = births
                    .values()
                    .filter(|t| **t >= now - 2 * window && **t < now - window)
                    .count();
                json!({"source":"git first addition per ID (all refs)","window_days":days,"current_new_questions":current,"previous_new_questions":previous,"current_per_day":current as f64 / days as f64,"previous_per_day":previous as f64 / days as f64,"change_per_day":(current as f64-previous as f64)/days as f64,"decay_fraction":if previous>0 { Some(1.0-current as f64/previous as f64) } else { None },"note":"Uncommitted questions are excluded from arrival rates"})
            }
            _ => {
                json!({"source":"git","available":false,"note":"Arrival rates are unavailable because git history could not be retrieved"})
            }
        };
        Ok(
            json!({"criteria":{"total":criteria.len(),"satisfied":satisfied,"items":criteria},"question_arrival":rate,"note":INTEGRITY_NOTE}),
        )
    }
}
