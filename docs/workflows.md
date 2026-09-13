# Configured workflow records

gy can require structured records at requirement states and explicitly submit records on configured node types. Configuration belongs to the ledger's `gy.toml`; an unconfigured ledger keeps its existing workflow.

gy checks recorded values and their relationships. It does not fetch URLs, inspect Git branches or PRs, run quality gates, or authenticate approvers. A recorded `passed` result is the caller's report.

## Start with the example

[workflow.toml](../crates/gy/examples/workflow.toml) supplies a profile for dispatch, research, design or design waiver, approval, implementation reports, quality gates, deviations, and audit records. [workflow-records.json](../crates/gy/examples/workflow-records.json) contains matching **illustrative data**, not evidence for a real project.

Merge the example's `workflow` tables into the ledger configuration, preserving existing settings. Edit the profile to match the project:

- The example requires research on every need and question. Change `kinds` or `required` if that is not your policy.
- Guards apply to every listed destination state, regardless of the previous state. Later states are listed too, so skipping directly to a later state cannot bypass required records.
- The example requires reports for gate failures, existing violations, and unexecuted or inapplicable gates. It does not require every result to be `passed`; the project decides which reported results prevent progress.
- Scope checks compare declared file lists exactly. File existence and completeness of the declared PR changes remain the caller's responsibility.
- Adding a schema does not change any node's state or fabricate past approvals.

Run `gy lint --json` after enabling a profile. Omissions on ongoing work appear under `workflow`. Completed requirements are historical records; adopting a profile does not retroactively require its forms. `gy handover --json` includes those diagnostics and the effective `workflow` configuration, so a new agent can discover the forms and guards.

## Define a record

Each record name is a top-level frontmatter attribute. This example requires a research report on needs:

```toml
[workflow.records.research]
description = "Evidence and limits supporting the recorded answer."
kinds = ["need"]
required = true
version_field = "revision"

[workflow.records.research.fields]
revision = { type = "string" }
question = { type = "string", description = "The question this research answers." }
population = { type = "integer" }
sources = { type = "array", items = { type = "string" } }
limitations = { type = "array", allow_empty = true, items = { type = "string" } }
location = { type = "string" }
```

`kinds` selects node types. `required = true` makes a missing record a lint finding for those types. Otherwise a record is required only by a state guard or explicit submission. On ongoing work, a present record is validated even outside those states. For completed requirements, passive inspection validates saved workflow history against its saved schemas; explicit submissions still validate against the current schema.

`version_field` is optional. It must name a required nonempty string field. After submission or a transition snapshot, the same version cannot be recorded with different contents. Change the revision when changing contents. This compares recorded data, not the remote document behind a URL.

| Field setting | Meaning |
| --- | --- |
| `description` | Optional project-authored explanation; never used for validation |
| `type` | `string`, `url`, `boolean`, `integer`, `number`, `array`, or `object` |
| `required` | Defaults to true; false permits an absent field |
| `allow_empty` | Defaults to false; permits empty strings, arrays, or objects when true |
| `values` | Allowed string values, compared exactly |
| `equals` | A required value, such as `false` for a reported out-of-scope change |
| `fields` | Nested object fields |
| `items` | Required schema for every array element |
| `variant_field` / `variants` | A discriminator and additional fields required for each alternative |

An optional field may be absent; null is not a valid substitute. Zero and false are valid numeric and boolean values. Unspecified attributes remain preserved. Configuration keys and field types are checked strictly to catch typos.

Record and field names use ASCII letters, digits, underscores, and hyphens. Core identity and lifecycle attributes cannot be record names. Check paths use dots for objects and `[]` to project array elements; literal dots in field names are not supported.

Records and fields (including nested fields, array items, and alternative fields) accept an optional string `description`. gy preserves it in the effective configuration returned by `handover --json` and in saved schemas. Descriptions explain project policy to readers; they do not change validation, comparisons, or record revisions. Existing configurations and saved schemas may omit them. To add explanations to an existing ledger, edit its `gy.toml`; no node or historical record migration is needed.

## Explicit absence and exceptions

```toml
[workflow.records.findings]
kinds = ["requirement"]

[workflow.records.findings.fields.report]
type = "object"
variant_field = "status"

[workflow.records.findings.fields.report.fields.status]
type = "string"

[workflow.records.findings.fields.report.variants.none]

[workflow.records.findings.fields.report.variants.not-applicable]
reason = { type = "string" }

[workflow.records.findings.fields.report.variants.not-executed]
reason = { type = "string" }

[workflow.records.findings.fields.report.variants.reported]
details = { type = "string" }
```

`{"status":"none"}` is an explicit report. `{"status":"not-executed"}` is incomplete until its reason is supplied. Unknown alternatives fail validation.

The full example uses this mechanism for `normal` and `design-waived`. Both keep a versioned work plan, contracts, and file scope. The normal route requires design details; the waiver requires a reason, location, and stopping conditions. Both require a separate approver, approval location, and matching target revision before implementation.

## Write, submit, and advance

Use `node set` to author a record. Drafts may be incomplete so they can be filled incrementally; lint exposes omissions.

```sh
gy node set N-1 --set 'research={"revision":"r1","question":"Can retries duplicate delivery?","population":2,"sources":["retry-test.log"],"limitations":[],"location":"research/retries.md"}'
gy node submit N-1 --record research --evidence "Reviewed retry-test.log"
```

`node submit` validates the selected record and appends its data, schema, timestamp, and evidence to `record_history`. It does not change state or submit unrelated records. MCP `gy_node` accepts the same arguments after `node`.

Requirement transitions use `gy req advance` as before:

```toml
[workflow.guards.start_implementation]
states = ["awaiting-implementation", "awaiting-audit"]
records = ["design_proposal", "approval"]

[[workflow.guards.start_implementation.checks]]
kind = "equal"
left = "approval.target_revision"
right = "design_proposal.revision"
```

This guard requires schemas named `design_proposal` and `approval`. All guards matching the destination apply, including when entering from another state or adding parenthesized context. Failure returns exit code 2 without changing the node or history. Invalid configuration returns 3.

A successful transition stores the applicable checks and all present, validated records and their schemas under `transitions[].workflow`, alongside the transition evidence. Current-state checks also run in lint and handover for ongoing work. Every explicit transition uses the current destination policy, including transitions from a completed requirement or back into `complete`. `[lint] workflow = "warn"` or `"off"` changes lint reporting only; it does not disable transition or submission guards.

## Revisions and approval

A design revision stays on the normal route. Update `design_proposal.revision` and the design record, move back to `awaiting-approval`, and record a new approval whose target matches before entering implementation again. The example keeps approval records versioned too.

Old snapshots retain the earlier design and approval. `node set` cannot replace `record_history` or `transitions`. Direct edits to canonical files remain possible; histories are records, not cryptographic proof of external approval.

URL contents are not monitored. If an external design changes, the caller must identify its new revision in the ledger. Use distinct, retained document versions or comments for traceability.

## Compare records

Checks compare records on the same node. Every root must be in that guard's `records`. Paths must resolve to declared fields. Paths through alternative-specific fields are unsupported; place shared comparison fields outside alternatives.

| Check | Meaning |
| --- | --- |
| `equal` | Two single values match exactly, including JSON types |
| `same-set` | Both operands contain the same set of nonempty strings |
| `subset` | Every value on the left is present on the right |

Sets ignore order and repeated values. Missing paths or wrong types fail. Empty collections are accepted only when the schema allows them. For scope, use `same-set` for exact file lists or `subset` with reported files on the left and allowed files on the right.

To check contract/gate associations, compare tuples:

```toml
[[workflow.guards.audit.checks]]
kind = "same-set"
left = "design_proposal.gates"
right = "quality_gates.results"
left_keys = ["contract", "id"]
right_keys = ["contract", "gate"]
```

This detects swapped contracts even if every gate and contract name appears somewhere in the report. The example also compares planned gates with dispatch requirements and checks that every changed contract has results.

gy cannot discover unreported contracts, files, failed tests, or primary sources. The caller must record the authoritative inputs.

## Compression and migration

Completion ends current workflow obligations. Lint and compression validate existing workflow snapshots against the schemas and comparisons stored in those snapshots, not against today's profile. A completed requirement with no workflow history does not acquire missing-history errors when a profile is introduced. This is a lifecycle rule, not an import exemption or a claim that old work satisfied the new policy.

Compression still requires valid completion records, the six retained fields, reviewed constraints, and a valid archive location when writing. The archive includes existing workflow data and histories. Compression removes `record_history` with `transitions` and the existing documented transient fields; other extension attributes remain preserved. Retrieve archived history from `compressed_from`. Compression is storage compaction, not a means to bypass current policy.

To adopt a profile, preserve completed records as they are, fill declarations needed for ongoing work from evidence, and use explicit submissions or subsequent transitions to record what is checked now. Do not reconstruct historical approvals to satisfy a policy that was not in force. Reopening completed work is a new action: current destination guards apply. New work cannot bypass completion guards by jumping directly to `complete`.

See [ledger semantics](architecture.md) for the shared authority and lifecycle model.
