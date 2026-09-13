//! Comparisons between declared record values.
use super::declared_files::match_declared_files;
use crate::*;
use serde_json::Value;
use std::collections::BTreeSet;

fn path_values<'a>(node: &'a Node, path: &str) -> Option<Vec<&'a Value>> {
    let mut parts = path.split('.');
    let root = node.attrs.get(parts.next()?)?;
    let mut values = vec![root];
    for part in parts {
        let expand = part.ends_with("[]");
        let key = part.strip_suffix("[]").unwrap_or(part);
        let mut next = vec![];
        for value in values {
            let value = value.get(key)?;
            if expand {
                next.extend(value.as_array()?.iter());
            } else {
                next.push(value);
            }
        }
        values = next;
    }
    Some(values)
}
fn string_set(values: Vec<&Value>, keys: &[String]) -> Option<BTreeSet<Vec<String>>> {
    let mut set = BTreeSet::new();
    for value in values {
        let items: Vec<_> = match value {
            Value::Array(a) => a.iter().collect(),
            v => vec![v],
        };
        for item in items {
            let values = if keys.is_empty() {
                vec![item]
            } else {
                keys.iter()
                    .map(|key| item.get(key))
                    .collect::<Option<Vec<_>>>()?
            };
            let tuple = values
                .into_iter()
                .map(|value| {
                    value
                        .as_str()
                        .filter(|s| !s.trim().is_empty())
                        .map(str::to_owned)
                })
                .collect::<Option<Vec<_>>>()?;
            set.insert(tuple);
        }
    }
    Some(set)
}
pub(super) fn check_records(node: &Node, checks: &[RecordCheck]) -> Vec<String> {
    let mut issues = vec![];
    for check in checks {
        let (Some(left), Some(right)) = (
            path_values(node, &check.left),
            path_values(node, &check.right),
        ) else {
            issues.push(format!(
                "Cannot compare {} and {}: a path is missing or has the wrong type",
                check.left, check.right
            ));
            continue;
        };
        let valid = match check.kind {
            CheckKind::MatchesDeclaredFiles => {
                let result = if check.left_keys.is_empty() && check.right_keys.is_empty() {
                    match_declared_files(&left, &right)
                } else {
                    Err("comparison keys are not supported".into())
                };
                if let Err(reason) = result {
                    issues.push(format!(
                        "MatchesDeclaredFiles check failed: {} versus {}: {reason}",
                        check.left, check.right
                    ));
                }
                continue;
            }
            CheckKind::Equal => {
                left.len() == 1 && right.len() == 1 && left == right && !left[0].is_null()
            }
            CheckKind::SameSet | CheckKind::Subset => match (
                string_set(left, &check.left_keys),
                string_set(right, &check.right_keys),
            ) {
                (Some(left), Some(right)) => match check.kind {
                    CheckKind::SameSet => left == right,
                    CheckKind::Subset => left.is_subset(&right),
                    _ => unreachable!(),
                },
                _ => false,
            },
        };
        if !valid {
            issues.push(format!(
                "{:?} check failed: {} versus {} (set operands must contain nonempty strings)",
                check.kind, check.left, check.right
            ));
        }
    }
    issues
}
