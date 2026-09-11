---
name: gy-ledger
description: Use when recovering work status from a gy ledger and updating needs, requirements, acceptance criteria, and gates.
---

# gy Ledger

Use gy to recover recorded progress and maintain the relationships that explain it. The ledger stores one node per Markdown file, with YAML frontmatter as the source of truth.

## Recover the current state

Start with:

```sh
gy cheatsheet
gy handover --json
gy next
```

Review the missing records reported by handover. Locate supporting evidence with `gy find` and `gy show <ID>`. The user selects from the needs listed by `next`; gy does not prioritize them.

## Choose nodes and scope

- N: need; Q: question; D: decision.
- #: requirement identified by GitHub Issue number.
- AC: acceptance criterion; G: continuation gate.

Reads cover all scopes. To create a node, enter its scope directory or supply `--scope`. Let gy allocate IDs and use `gy link` to save relationships on both sides. Unknown attributes survive updates and are searchable with `gy find --where key=value`.

## Record work and transitions

Every need must support an acceptance criterion through `targets`. Before `gy need file`, verify the parent Issue on GitHub and record it in the requirement's `parent_issue`. gy does not query GitHub.

For active requirements, maintain:

- `next_evidence`: evidence needed for the next transition.
- `responsible`: the responsible party.
- `waiting-on`: an array of question IDs referenced as unresolved.

Read `gy req advance --help` for transition guards, then supply the verification results through `--evidence`. At completion, record design deviations in `deviations` and transferred work destinations in `residual`. Use `none` explicitly when absent; transfer work to existing N / Q / # IDs before completing the requirement.

## Submit configured records

Read `workflow` in `gy handover --json` for this ledger's forms and guards. Author records with `gy node set`; use `gy node submit <ID> --record <name> --evidence "<record>"` to validate and snapshot a report without advancing state.

For requirement transitions, gy validates the destination guards and saves the checked inputs. Give a changed design a new revision and record approval for that revision. Report absence, inapplicability, and non-execution using the configured alternatives; a blank field is not an explicit report.

## Compress a completed requirement

Read `gy req compress --help` for the six retained fields: `summary`, `contracts_changed`, `artifacts`, `production`, `deviations`, and `residual`.

1. Associate each `constraints` entry with a decision that states its applicability conditions. Add `relies-on` links and record `constraints_reviewed=true`.
2. Run `gy req compress <issue>` to obtain the full original record.
3. Archive that text in the corresponding Issue comment and verify its contents. If the source changed, archive the latest full text again.
4. Pass the archive URL with `gy req compress <issue> --evidence <comment-URL>`. gy rechecks the record, compresses the body, and stores the URL in `compressed_from`.

Full quality gates, design proposals, and audit records remain in the archive. Acceptance criteria, questions, and decisions remain reachable through the graph.

## Verify the result

Run `gy lint` and inspect the affected nodes with `gy show`. A successful lint result establishes internal graph consistency. People and agents separately verify the ledger against code, GitHub, and production, and review whether options and applicability conditions are meaningful.
