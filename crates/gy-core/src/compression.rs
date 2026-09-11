use crate::*;
use serde_json::Value;

pub const COMPRESSION_FIELDS: &[(&str, &str)] = &[
    ("summary", "One-sentence summary"),
    ("contracts_changed", "Changed contracts"),
    ("artifacts", "Deliverable locations"),
    ("production", "Production measurements"),
    (
        "deviations",
        "Deviations from the approved design and additional decisions",
    ),
    ("residual", "Destinations for remaining work"),
];

fn nonempty(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(s) => !s.trim().is_empty(),
        Value::Array(a) => !a.is_empty() && a.iter().all(nonempty),
        Value::Object(m) => !m.is_empty() && m.values().all(nonempty),
        _ => true,
    }
}

/// A residual list is an explicit transfer of work, not unfinished work in this
/// requirement. Free text alone cannot identify the new owner of that work.
pub fn residual_destinations(n: &Node) -> Result<Vec<String>> {
    let Some(value) = n.attrs.get("residual") else {
        return Err(Error::input(
            "residual is missing. Record destinations for remaining work, or explicitly use 'none'",
        ));
    };
    if value.as_str() == Some("none") {
        return Ok(vec![]);
    }
    let values = match value {
        Value::Array(a) if !a.is_empty() => a.iter().collect::<Vec<_>>(),
        Value::String(_) | Value::Object(_) => vec![value],
        _ => vec![],
    };
    if values.is_empty() {
        return Err(Error::input(
            "residual must be 'none' or a nonempty list of destination IDs N-xx / Q-xx / #Issue. Blank values and empty arrays do not record a review",
        ));
    }
    let mut out = vec![];
    for value in values {
        let id = value
            .as_str()
            .or_else(|| value.get("id").and_then(Value::as_str))
            .unwrap_or("");
        if !["need", "question", "requirement"]
            .iter()
            .any(|kind| valid_id(id, kind))
            || id == n.id()
        {
            return Err(Error::input(format!(
                "residual contains work without an identifiable destination: {value}. Transfer it to another N-xx / Q-xx / #Issue"
            )));
        }
        out.push(id.into());
    }
    Ok(out)
}

impl Store {
    /// Used both at transitions and by L11, so editing frontmatter cannot make
    /// an absent completion record appear equivalent to an explicit 'none'.
    pub fn completion_issues(&self, n: &Node) -> Vec<String> {
        let mut issues = vec![];
        if !n.attrs.get("deviations").is_some_and(|v| {
            matches!(v, Value::String(_) | Value::Array(_) | Value::Object(_)) && nonempty(v)
        }) {
            issues.push("deviations is missing or blank. Record deviations from the approved design and additional decisions, or explicitly use 'none'".into());
        }
        match residual_destinations(n) {
            Err(e) => issues.push(e.message),
            Ok(ids) => {
                for id in ids {
                    match self.nodes.get(&id) {
                    Some(target) if ["need", "question", "requirement"].contains(&target.kind()) => (),
                    _ => issues.push(format!("residual destination {id} does not exist. Create the destination with gy before completing the requirement")),
                }
                }
            }
        }
        if n.attrs.contains_key("remaining_work") && legacy_remaining(n) != Some(0) {
            issues.push("remaining_work contains unfinished work or an invalid value. Check against residual destinations and reduce this requirement's remaining work to zero".into());
        }
        if n.attrs.get("production").and_then(Value::as_str) == Some("none")
            && ["data_migration", "production_only"]
                .iter()
                .any(|key| n.attrs.get(*key).and_then(Value::as_bool) == Some(true))
        {
            issues.push("Production work is recorded as required. Record measurements, the target environment, and verification results instead of production=none".into());
        }
        if let Some(date) = n.attrs.get("compressed") {
            if date
                .as_str()
                .is_none_or(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_err())
            {
                issues.push("Record the compression date in compressed as YYYY-MM-DD".into());
            }
            if !archive_comment_matches(n.id(), n.get("compressed_from")) {
                issues.push(
                    "compressed_from is missing the corresponding Issue comment URL containing the archived full text".into(),
                );
            }
        }
        issues
    }
    pub fn compression_field_issues(&self, n: &Node) -> Vec<String> {
        let mut issues = self.completion_issues(n);
        if n.attrs
            .get("summary")
            .and_then(Value::as_str)
            .is_none_or(|s| s.trim().is_empty() || s.contains(['\n', '\r']))
        {
            issues.push("Record the outcome in summary as one sentence on one line".into());
        }
        let contracts = n.attrs.get("contracts_changed");
        if !contracts.is_some_and(|v| {
            v.as_str().is_some_and(|s| !s.trim().is_empty())
                || v.as_array().is_some_and(|a| {
                    !a.is_empty()
                        && a.iter()
                            .all(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()))
                })
        }) {
            issues.push("Record changed APIs, data formats, processing paths, or schemas in contracts_changed as free-form text or a list".into());
        }
        match n.attrs.get("artifacts").and_then(Value::as_object) {
            Some(a) => {
                if a.get("pr").and_then(Value::as_str).is_none_or(|s| !valid_url(s) || !s.contains("/pull/")) { issues.push("Record the PR URL in artifacts.pr".into()); }
                if a.get("merge_commit").and_then(Value::as_str).is_none_or(|s| !(7..=64).contains(&s.len()) || !s.bytes().all(|b| b.is_ascii_hexdigit())) { issues.push("Record the merge commit hash in artifacts.merge_commit (7–64 hexadecimal digits)".into()); }
                if a.get("base_branch").and_then(Value::as_str).is_none_or(|s| s.trim().is_empty()) { issues.push("Record the base branch in artifacts.base_branch".into()); }
            },
            None => issues.push("Record pr / merge_commit / base_branch in artifacts so deliverables can be located through git".into()),
        }
        if !n.attrs.get("production").is_some_and(|v| {
            v.as_str() == Some("none")
                || v.as_object()
                    .is_some_and(|m| !m.is_empty() && m.values().all(nonempty))
        }) {
            issues.push("Record production measurements, the target environment, and verification results in production. Explicitly use 'none' if no production work was needed".into());
        }
        issues
    }
    pub fn compress(&mut self, id: &str, evidence: &str) -> Result<String> {
        let archive = self.compression_preview(id)?;
        let issue = id.trim_start_matches('#');
        let valid_comment = archive_comment_matches(id, evidence);
        if !valid_comment {
            return Err(Error::input(format!(
                "Set --evidence to the Issue comment URL containing the archived full text of {id} (.../issues/{issue}#issuecomment-number). The caller must verify that the archive exists and contains the full record"
            )));
        }
        let node = self.nodes.get_mut(id).unwrap();
        let body = compressed_body(node)?;
        // Only documented transient fields are removed. Identity, graph edges,
        // the six searchable records and user extensions remain in frontmatter.
        for key in [
            "constraints",
            "constraints_reviewed",
            "remaining_work",
            "transitions",
            "next_evidence",
            "responsible",
            "pr_url",
            "pr_base",
            "pr_files",
            "data_migration",
            "production_only",
            "production_done",
            "cleanup_done",
            "evidence",
            "quality_gates",
            "design_proposal",
            "audit_records",
        ] {
            node.attrs.remove(key);
        }
        node.put(
            "compressed",
            chrono::Utc::now().format("%Y-%m-%d").to_string(),
        );
        node.put("compressed_from", evidence);
        node.body = body;
        Ok(archive)
    }
}

pub fn legacy_remaining(n: &Node) -> Option<u64> {
    match n.attrs.get("remaining_work") {
        Some(Value::Array(a)) => Some(a.len() as u64),
        Some(v) => v.as_u64(),
        None => None,
    }
}

/// Build the human-readable six-item view from the canonical frontmatter.
pub fn compressed_body(node: &Node) -> Result<String> {
    let mut body = String::new();
    for (index, (key, heading)) in COMPRESSION_FIELDS.iter().enumerate() {
        body.push_str(&format!("\n## {}. {heading}\n\n", index + 1));
        let value = node.attrs.get(*key).unwrap_or(&Value::Null);
        match value {
            Value::String(s) => body.push_str(s),
            _ => {
                let yaml =
                    serde_yaml::to_string(value).map_err(|e| Error::corrupt(e.to_string()))?;
                let longest = yaml
                    .lines()
                    .map(|line| line.chars().take_while(|c| *c == '`').count())
                    .max()
                    .unwrap_or(0);
                let fence = "`".repeat(3.max(longest + 1));
                body.push_str(&format!("{fence}yaml\n{yaml}{fence}"));
            }
        }
        body.push('\n');
    }
    Ok(body)
}

fn archive_comment_matches(id: &str, evidence: &str) -> bool {
    let issue = id.trim_start_matches('#');
    evidence.split_once('#').is_some_and(|(url, fragment)| {
        valid_url(url)
            && url.ends_with(&format!("/issues/{issue}"))
            && fragment
                .strip_prefix("issuecomment-")
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    })
}
