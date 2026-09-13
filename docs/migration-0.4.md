# Migrating to gy 0.4

Existing configurations and recorded history remain readable by 0.4. Keep existing profiles if their exact-string comparisons meet the project's needs. The new file comparison is opt-in. Back up the ledger before adopting a new profile; do not rewrite historical snapshots or archived evidence.

## Update workflow profiles and records

1. Upgrade every reader of the ledger before adding `description` or `matches-declared-files`. Older binaries reject descriptions in configuration and saved schemas, and do not recognize the new comparison in configuration or history. Removing a description from today's configuration does not remove descriptions already saved in history. Keep an earlier ledger backup if an older reader is required.
2. To document record or field meanings, add optional string `description` entries to `gy.toml`. Discover them in `gy handover --json`. Descriptions do not change validation or recorded versions. Existing records need no edits for this feature.
3. For successful gates with no population concept, add `passed-without-population = { reason = { type = "string" }, evidence = { type = "string" } }` to `workflow.records.quality_gates.fields.results.items.fields.result.variants`. Preserve `passed` and its required population, denominator, and evidence, plus local variants and guards. Use the new variant only for gates without a population concept; a missing measurement does not qualify. This configuration addition can be repeated by replacing the same entry; it must not append duplicate TOML keys.
4. To adopt generated file declarations, copy the `design_proposal.fields.files` schema from the [minimal profile](../crates/gy/examples/file-scope.toml), preserving the rest of the project's design schema. Replace the relevant file comparison with `kind = "matches-declared-files"`, `left = "implementation_report.files"`, and `right = "design_proposal.files"`. Remove comparison keys or `[]` projections from that comparison. Keep implementation files as an array of concrete strings. The [full example](../crates/gy/examples/workflow.toml) shows the combined guards.
5. For each ongoing design that adopts the new shape, replace a string such as `"src/retry.rs"` with `{"kind":"literal","path":"src/retry.rs"}`. Replace a not-yet-generated name with a `generated` declaration containing its fixed directory, prefix/suffix, token class and length. The [illustrative records](../crates/gy/examples/workflow-records.json) demonstrate both. Only convert current declarations supported by the actual design; do not infer a generated shape from an old placeholder automatically.
6. If the old design was already submitted or captured by a transition, record a new design revision and its matching approval through the project's normal approval process. Update reports that refer to that design revision as appropriate. Existing saved versions cannot be reused with different data. Keep old snapshots unchanged. Once the generated shape is part of the approved design, reporting its concrete name does not require changing that shape or design revision.
7. Run `gy lint --json` and inspect `gy handover --json`. In-progress records may need updates under the adopted profile; completed records still use their saved schemas and comparisons. New submissions and transitions apply today's profile. A successful check verifies reported consistency, not file existence, execution results, or approval identity.

## Update Rust clients

`DependencyRole`, `RuleConfig`, `FieldType`, and `CheckKind` are now non-exhaustive enums. A downstream `match` must include a wildcard arm for future variants. `CheckKind` also adds `MatchesDeclaredFiles`. Select a deliberate fallback; the example below returns an error for unknown variants.

Known unit, tuple, and struct variants remain constructible. For example, `RuleConfig::Detail { enabled: Some(true), severity: None }` is still allowed. This differs from the non-exhaustive **structs** introduced in 0.3: downstream struct literals for those types remain forbidden. `RecordSchema` and `FieldSchema` still have no `Default`; construct them through deserialization. No new constructors are required for the enums.

Create a separate crate with these dependencies. Before publication, replace the `gy-core` dependency with a local path to the checked-out `crates/gy-core` while verifying the same source.

```toml
[package]
name = "gy-migration-check"
version = "0.1.0"
edition = "2024"

[dependencies]
gy-core = "0.4.0"
serde_json = "1"
toml = "0.8"
tempfile = "3"
```

Save the following as `src/main.rs`. Copy the [minimal file-scope profile](../crates/gy/examples/file-scope.toml) to this crate's `gy.toml` and the [shared baseline requirement](../crates/gy/tests/fixtures/requirement.md) to `requirement.md`, then run `cargo run -- gy.toml requirement.md`. Retain the resulting lockfile and use `cargo run --locked -- gy.toml requirement.md` to repeat with the same resolved dependencies. This program creates only a temporary test ledger, validates the same baseline requirement used by the integration tests, reads the supplied configuration, checks a matching report, and confirms that an extra file is rejected.

```rust
use gy_core::{CheckKind, Config, DependencyRole, FieldType, Node, RuleConfig, Store};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let role = DependencyRole::Current;
    let _role_name = match role {
        DependencyRole::Current => "current",
        DependencyRole::Historical => "historical",
        _ => return Err("unsupported dependency role".into()),
    };
    let rule = RuleConfig::Detail { enabled: Some(true), severity: None };
    let _rule_name = match rule {
        RuleConfig::Enabled(_) => "enabled",
        RuleConfig::Severity(_) => "severity",
        RuleConfig::Detail { .. } => "detail",
        _ => return Err("unsupported rule configuration".into()),
    };
    let field = FieldType::Array;
    let _field_name = match field {
        FieldType::String => "string",
        FieldType::Url => "url",
        FieldType::Boolean => "boolean",
        FieldType::Integer => "integer",
        FieldType::Number => "number",
        FieldType::Array => "array",
        FieldType::Object => "object",
        _ => return Err("unsupported field type".into()),
    };
    let path = std::env::args().nth(1).ok_or("supply the minimal gy.toml path")?;
    let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
    config.workflow.validate()?;
    let check = config.workflow.guards["file_scope"].checks[0].kind;
    let check_name = match check {
        CheckKind::Equal => "equal",
        CheckKind::SameSet => "same-set",
        CheckKind::Subset => "subset",
        CheckKind::MatchesDeclaredFiles => "matches-declared-files",
        _ => return Err("unsupported comparison".into()),
    };
    assert_eq!(check_name, "matches-declared-files");
    let temp = tempfile::tempdir()?;
    let mut store = Store::init(temp.path(), "test", None)?;
    let fixture = std::env::args().nth(2).ok_or("pass the baseline requirement path")?;
    let mut node = Node::parse(
        &std::fs::read_to_string(fixture)?,
        store.root.join("test/requirements/1.md"),
    )?;
    store.nodes.insert(node.id().to_owned(), node.clone());
    assert!(store.lint(None).is_empty());
    store.config = config;
    node.put("status", "awaiting-audit");
    node.put("design_proposal", json!({"revision":"d1", "files":[
        {"kind":"literal", "path":"src/retry.rs"},
        {"kind":"generated", "directory":"db/migrate", "prefix":"",
         "suffix":"_add_retry_keys.rb", "token_class":"ascii-digits", "token_length":14}
    ]}));
    node.put("implementation_report", json!({"files":[
        "src/retry.rs", "db/migrate/20260913090000_add_retry_keys.rb"
    ]}));
    assert!(store.workflow_issues(&node, Some("awaiting-audit")).is_empty());
    node.attrs["implementation_report"]["files"].as_array_mut().unwrap()
        .push(json!("undeclared.rs"));
    let issues = store.workflow_issues(&node, Some("awaiting-audit"));
    assert!(issues.iter().any(|s| s.contains("undeclared.rs")));
    println!("enum matches, profile loading, declared file acceptance and rejection passed");
    Ok(())
}
```
