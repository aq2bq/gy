//! Single-file HTML projection of the ledger.
//!
//! The generated page is a derived, read-only view. All judgments shown on
//! the page (lint, next, handover facts, stats, decision dependencies) come
//! from the same core functions the CLI uses; the page never recomputes a
//! `lint` or `next` decision itself. Filters and search over embedded data
//! are display-time selection, not judgment.

use crate::*;
use serde_json::{Value, json};

// Preserve source order and bytes inside one shared JavaScript closure.
const TEMPLATE: &str = concat!(
    include_str!("html/head.html"),
    include_str!("html/style.css"),
    include_str!("html/body.html"),
    include_str!("html/data.js"),
    include_str!("html/summary.js"),
    include_str!("html/controls.js"),
    include_str!("html/state.js"),
    include_str!("html/layout.js"),
    include_str!("html/svg.js"),
    include_str!("html/selection.js"),
    include_str!("html/draw.js"),
    include_str!("html/navigation.js"),
    include_str!("html/clusters.js"),
    include_str!("html/viewport.js"),
    include_str!("html/detail.js"),
    include_str!("html/blockers.js"),
    include_str!("html/progress.js"),
    include_str!("html/init.js"),
    include_str!("html/tail.html"),
);
const ARRIVAL_DAYS: u32 = 7;

/// Escape a JSON document so it can be embedded as a JavaScript object
/// literal without terminating the enclosing `</script>` tag.
fn escape_json_for_js(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

fn node_payload(n: &Node, superseded: &std::collections::BTreeMap<String, Vec<String>>) -> Value {
    let state = if n.kind() == "requirement" {
        base_state(n.get("status")).map(str::to_owned)
    } else {
        None
    };
    // Use the same reverse declarations and exact anchors as display_node.
    let body_marks: Vec<Value> = [
        ("narrowed-by", "Applicability narrowed"),
        ("superseded-by", "Superseded"),
    ]
    .into_iter()
    .flat_map(|(key, label)| {
        n.refs(key).into_iter().map(move |source| {
            let mark = edge_mark(n, key, &source);
            let found = mark.as_ref().is_some_and(|text| n.body.contains(text));
            json!({"source": source, "label": label, "mark": mark, "found": found})
        })
    })
    .collect();
    json!({
        "body_marks": body_marks,
        "id": n.id(),
        "type": n.kind(),
        "scope": n.scope(),
        "title": n.get("title"),
        "status": n.get("status"),
        "closed": n.closed(),
        "state": state,
        "created": n.get("created"),
        "superseded_by": superseded.get(n.id()).cloned().unwrap_or_default(),
        "attrs": n.attrs,
        "body": n.body,
    })
}

impl Store {
    fn html_payload(&self, scope: Option<&str>) -> Result<Value> {
        let scoped: Vec<_> = self
            .nodes
            .values()
            .filter(|n| scope.is_none_or(|s| n.scope() == s))
            .collect();

        // superseded_by per decision, collected from both edge directions, so
        // a broken inverse link cannot hide supersession (same contract as
        // DecisionDependency and the `edges` rule).
        let mut superseded: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for n in &scoped {
            if n.kind() != "decision" {
                continue;
            }
            let mut successors: std::collections::BTreeSet<String> =
                n.refs("superseded-by").into_iter().collect();
            for other in &scoped {
                if other.kind() == "decision"
                    && other.refs("supersedes").contains(&n.id().to_owned())
                {
                    successors.insert(other.id().to_owned());
                }
            }
            if !successors.is_empty() {
                superseded.insert(n.id().to_owned(), successors.into_iter().collect());
            }
        }

        // Forward edges only, same direction extraction as `dot`, so each
        // logical relationship is drawn once and never duplicates its reverse.
        let mut edges: Vec<Value> = Vec::new();
        for n in &scoped {
            for (label, source_kind, _, reverse) in EDGES {
                if !kind_matches(n.kind(), source_kind) {
                    continue;
                }
                for target in n.refs(label) {
                    if scoped.iter().any(|m| m.id() == target) {
                        edges.push(json!({
                            "source": n.id(),
                            "target": target,
                            "label": label,
                            "reverse": reverse,
                        }));
                    }
                }
            }
        }
        edges.sort_by_key(|e| {
            (
                e["source"].as_str().unwrap_or("").to_owned(),
                e["label"].as_str().unwrap_or("").to_owned(),
            )
        });

        let dependencies: Vec<Value> = scoped
            .iter()
            .filter(|n| n.kind() == "requirement")
            .flat_map(|n| self.decision_dependencies(n))
            .map(|d| {
                json!({
                    "requirement": d.requirement,
                    "decision": d.decision,
                    "role": match d.role {
                        DependencyRole::Current => "current",
                        DependencyRole::Historical => "historical",
                    },
                    "superseded_by": d.superseded_by,
                })
            })
            .collect();

        let open_questions: Vec<Value> = scoped
            .iter()
            .filter(|n| n.kind() == "question" && !n.closed())
            .map(|n| {
                let referencing = self.waiting_references(n.id());
                json!({
                    "id": n.id(),
                    "scope": n.scope(),
                    "title": n.get("title"),
                    "referencing": referencing,
                })
            })
            .collect();

        let next: Vec<Value> = self
            .next(scope)
            .iter()
            .map(|n| json!({"id": n.id(), "label": n.get("title")}))
            .collect();

        let missing: Vec<Value> = scoped
            .iter()
            .filter(|n| n.kind() == "requirement" && !n.is_complete_requirement())
            .filter(|n| {
                n.get("next_evidence").trim().is_empty() || n.get("responsible").trim().is_empty()
            })
            .map(|n| {
                json!({
                    "id": n.id(),
                    "next_evidence": n.get("next_evidence"),
                    "responsible": n.get("responsible"),
                })
            })
            .collect();

        let dangling: Vec<Value> = scoped
            .iter()
            .flat_map(|n| {
                n.attrs
                    .iter()
                    .filter(|(k, _)| k.as_str() != "id")
                    .flat_map(|(k, v)| recorded_attribute_ids(k, v))
                    .filter(|id| !self.nodes.contains_key(id))
                    .map(move |id| json!({"source": n.id(), "target": id, "label": format!("references missing node {id}")}))
                    .collect::<Vec<_>>()
            })
            .collect();

        let states: std::collections::BTreeMap<String, usize> = {
            let mut m: std::collections::BTreeMap<String, usize> =
                STATES.iter().map(|s| (s.to_string(), 0)).collect();
            for n in &scoped {
                if n.kind() == "requirement" {
                    if let Some(s) = base_state(n.get("status")) {
                        *m.entry(s.to_string()).or_default() += 1;
                    }
                }
            }
            m
        };

        let criteria: Vec<_> = scoped
            .iter()
            .filter(|n| n.kind() == "criterion")
            .map(|n| {
                json!({
                    "id": n.id(),
                    "title": n.get("title"),
                    "satisfied": n.attrs.get("satisfied").and_then(Value::as_bool).unwrap_or(false),
                    "satisfied_at": n.attrs.get("satisfied_at").cloned().unwrap_or(Value::Null),
                })
            })
            .collect();
        let total = criteria.len();
        let satisfied = criteria.iter().filter(|c| c["satisfied"] == true).count();

        // Primary progress axis: cumulative satisfied count over time,
        // built from the same `satisfied_at` records `stats` reports.
        let mut satisfied_dates: Vec<(String, i64, usize)> = Vec::new();
        for c in &criteria {
            if c["satisfied"] != true {
                continue;
            }
            if let Some(s) = c["satisfied_at"].as_str() {
                let dt = chrono::DateTime::parse_from_rfc3339(s)
                    .map(|d| d.timestamp())
                    .unwrap_or(0);
                satisfied_dates.push((s.to_string(), dt, 1));
            } else {
                satisfied_dates.push((String::new(), 0, 1));
            }
        }
        satisfied_dates.sort_by_key(|(_, ts, _)| *ts);
        let mut timeline: Vec<Value> = Vec::new();
        let mut run = 0usize;
        for (_, _, n) in &satisfied_dates {
            run += n;
            timeline.push(json!({"label": run.to_string(), "value": run}));
        }
        if timeline.is_empty() {
            timeline.push(json!({"label": "0", "value": 0}));
        }

        let arrival = self.stats(scope, ARRIVAL_DAYS)?;
        let arrival_value = arrival["question_arrival"].clone();
        let arrival_available = !arrival_value
            .get("available")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let scopes: Vec<String> = {
            let mut s: std::collections::BTreeSet<String> =
                scoped.iter().map(|n| n.scope().to_owned()).collect();
            s.extend(self.config.scopes.keys().cloned());
            s.into_iter().collect()
        };

        Ok(json!({
            "generated_at": today(),
            "scope": scope.map(str::to_owned),
            "scopes": scopes,
            "node_count": scoped.len(),
            "integrity_note": INTEGRITY_NOTE,
            "nodes": scoped.iter().map(|n| node_payload(n, &superseded)).collect::<Vec<_>>(),
            "edges": edges,
            "dependencies": dependencies,
            "open_questions": open_questions,
            "next": next,
            "missing": missing,
            "dangling": dangling,
            "states": states,
            "criteria": json!({"satisfied": satisfied, "total": total}),
            "criteria_timeline": timeline,
            "arrival": json!({
                "available": arrival_available,
                "current_per_day": arrival_value.get("current_per_day").and_then(Value::as_f64).unwrap_or(0.0),
                "previous_per_day": arrival_value.get("previous_per_day").and_then(Value::as_f64).unwrap_or(0.0),
                "decay_fraction": arrival_value.get("decay_fraction").cloned().unwrap_or(Value::Null),
            }),
            "lint": self.lint(scope),
        }))
    }

    /// Build the self-contained HTML projection for one output file.
    pub fn render_html(&self, payload_scope: Option<&str>) -> Result<String> {
        let payload = self.html_payload(payload_scope)?;
        let json = serde_json::to_string(&payload).unwrap();
        let escaped = escape_json_for_js(&json);
        Ok(TEMPLATE.replace("__PAYLOAD__", &escaped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Extract the embedded payload and the exact script-terminating region.
    fn extract_payload(html: &str) -> Value {
        let start = html.find("window.GY_DATA = ").unwrap() + "window.GY_DATA = ".len();
        let end = html[start..]
            .find(";\n</script>")
            .map(|i| start + i)
            .unwrap();
        let payload = &html[start..end];
        // The payload itself must not contain the script terminator sequence,
        // otherwise the data block would end early and break the page.
        assert!(!payload.contains("</script>"));
        assert!(serde_json::from_str::<Value>(payload).is_ok());
        serde_json::from_str(payload).unwrap()
    }

    #[test]
    fn body_marks_keep_exact_locations_and_do_not_infer_missing_anchors() {
        let mut node = Node::new("D-1", "decision", "Original", "a");
        node.body = "Keep this exact passage.\n".into();
        node.put(
            "superseded-by",
            json!([
                {"id":"D-2", "mark":"exact passage"},
                {"id":"D-3", "mark":"Exact passage"},
                {"id":"D-4", "mark":" "},
                "D-5"
            ]),
        );
        let payload = node_payload(&node, &Default::default());
        assert_eq!(payload["body"], node.body);
        let marks = payload["body_marks"].as_array().unwrap();
        assert_eq!(marks.len(), 4);
        assert_eq!(marks[0]["found"], true);
        assert_eq!(marks[0]["source"], "D-2");
        assert_eq!(marks[1]["found"], false);
        assert_eq!(marks[1]["mark"], "Exact passage");
        assert_eq!(marks[2]["mark"], Value::Null);
        assert_eq!(marks[3]["mark"], Value::Null);
        assert_eq!(node.body, "Keep this exact passage.\n");
    }

    fn store(cwd: &Path) -> Store {
        let mut s = Store::init(cwd, "a", None).unwrap();
        let id = s.create("criterion", "Criterion", "a", None).unwrap();
        s.nodes.get_mut(&id).unwrap().put("satisfied", true);
        let _ = s.add_need("a", "Need", &[id], None).unwrap();
        let _ = s
            .add_question(
                "a",
                "Question",
                QuestionOptions {
                    decider: "master",
                    options: &["A".into(), "B".into()],
                    bundle: None,
                    rationale: None,
                    force: false,
                },
            )
            .unwrap();
        s.commit().unwrap();
        s
    }

    #[test]
    fn payload_round_trips_body_byte_exactly() {
        let t = tempfile::tempdir().unwrap();
        let mut s = store(t.path());
        let id = s.create("decision", "D", "a", None).unwrap();
        let body = "</script><p>html</p><!-- comment -->\n\u{2028}\u{2029}end\u{2028}";
        s.nodes.get_mut(&id).unwrap().body = body.into();
        let html = s.render_html(None).unwrap();
        let decoded = extract_payload(&html);
        let round = decoded["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == "D-1")
            .unwrap();
        assert_eq!(round["body"], json!(body));
    }

    #[test]
    fn html_is_single_file_and_has_no_external_reference() {
        let t = tempfile::tempdir().unwrap();
        let mut s = store(t.path());
        let id = s
            .create("decision", "URL-bearing record", "a", None)
            .unwrap();
        s.nodes
            .get_mut(&id)
            .unwrap()
            .attrs
            .insert("pr_url".into(), json!("https://example.com/pull/1"));
        let html = s.render_html(None).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("https://example.com/pull/1"));
        // A coarse guard against literal resource references, not an HTML/JS
        // parser. URLs in payloads and code that builds user-followed links are
        // valid. E2E checks attempted HTTP(S) requests during load and interaction;
        // that runtime check enforces the no-external-fetch contract.
        for tok in [
            "src=\"http",
            "src='http",
            "href=\"http",
            "href='http",
            "@import",
            "url(http",
            "<link",
        ] {
            assert!(!html.contains(tok), "external reference {tok} present");
        }
        assert!(!html.contains("__PAYLOAD__"));
    }

    #[test]
    fn payload_has_all_required_keys() {
        let t = tempfile::tempdir().unwrap();
        let s = store(t.path());
        let p = s.html_payload(None).unwrap();
        for key in [
            "generated_at",
            "scopes",
            "node_count",
            "integrity_note",
            "nodes",
            "edges",
            "dependencies",
            "open_questions",
            "next",
            "missing",
            "dangling",
            "states",
            "criteria",
            "criteria_timeline",
            "arrival",
            "lint",
        ] {
            assert!(p.get(key).is_some(), "missing key {key}");
        }
        assert_eq!(p["node_count"], p["nodes"].as_array().unwrap().len() as u64);
    }

    #[test]
    fn scope_filter_includes_only_that_scope() {
        let t = tempfile::tempdir().unwrap();
        let mut s = store(t.path());
        let _ = s.create("need", "second", "a", None).unwrap();
        let scoped = s.render_html(Some("a")).unwrap();
        s.commit().unwrap();
        let payload = extract_payload(&scoped);
        assert_eq!(
            payload["node_count"],
            payload["nodes"].as_array().unwrap().len() as u64
        );
    }

    #[test]
    fn superseded_by_marks_only_the_superseded_decision() {
        let t = tempfile::tempdir().unwrap();
        let mut s = Store::init(t.path(), "a", None).unwrap();
        let d1 = s.create("decision", "old", "a", None).unwrap();
        let d2 = s.create("decision", "new", "a", None).unwrap();
        // D-1 supersedes D-2 twice; the second is a duplicate declaration.
        s.link(&d1, "supersedes", &d2, Some("passage")).unwrap();
        s.link(&d1, "supersedes", &d2, Some("passage")).unwrap();
        let payload = s.html_payload(None).unwrap();
        let nodes = payload["nodes"].as_array().unwrap();
        let sup = |id: &str| {
            nodes.iter().find(|n| n["id"] == id).unwrap()["superseded_by"]
                .as_array()
                .unwrap()
                .clone()
        };
        // The superseding decision is not itself marked.
        assert!(sup(&d1).is_empty());
        // The target is superseded by the source; duplicates collapse.
        let got = sup(&d2);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], json!(d1.clone()));
    }
}
