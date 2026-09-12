# Ledger semantics and derived views

## Authority

The canonical node file contains two different kinds of information:

- Frontmatter declares node identity, lifecycle state, relationships, and structured records. Core rules and configured workflow schemas define the meaning of those declarations.
- The Markdown body explains decisions, supplies context, and quotes evidence or code. Its vocabulary does not declare graph relationships, lifecycle states, or counts.

A consistency check evaluates declared records. It must not silently substitute a guess from prose when a declaration is absent. In particular, `waiting_checkout?`, a sentence saying a question is unresolved, and a quotation of an old report cannot create a `waiting-on` edge. Missing declarations cannot be detected by pretending that a prose inference is a declaration.

Body search remains available through `find`. Import may explicitly extract a configured section into an attribute; the import result is persisted and reported before later checks read it. `mark` is an explicit locator into body text: display and link warnings may inspect the body to locate it, without inferring a new relationship. Compression archives original records and generates its summary from declared attributes.

Unknown extension attributes are preserved. The existing L13 convention treats exact node-ID-valued attributes as references; it does not extract references from arbitrary prose strings. Workflow snapshot schemas are configuration, while their recorded values and evidence remain data.

## Scope relabeling

A scope is a label, not a node identity. Its canonical representations are the directory name, each member node's `scope` attribute, and the `[scopes]` entry in `gy.toml`; loading requires the directory and attribute to agree. `scope rename` changes all three together, so node IDs, relationships, lifecycle state, records, and history are preserved. Relationships are keyed by node ID and never embed a scope name, so cross-scope edges survive unchanged.

Renaming rejects a destination that already exists or a name that is not a safe directory component. A collision is not a merge. Free text, `decision_scope`, and arbitrary attributes that mention the old name are author declarations rather than scope references, so rename does not rewrite them; later users verify any such reference. The operation applies the new files and the old directory removal in one transaction, so an interrupted update replays to a consistent state.

## Dependency meaning and lifecycle

`requirement --relies-on--> decision` records the decision on which that requirement's work was based. The edge remains part of the ledger after completion or supersession. `supersedes` declares that a decision has replaced another decision; it does not retroactively replace the decisions used by earlier work.

The role of a requirement's decision dependencies is derived from its declared lifecycle:

| Requirement state | Dependency role | Superseded target |
| --- | --- | --- |
| Any recognized state except `complete` | Current work dependency | L5 reports the conflict; `next` does not offer work blocked by it |
| `complete`, including parenthesized context | Historical basis of completed work | Preserve and display it as history, not an L5 conflict |
| Missing or invalid state | Treated as current for dependency checks | L10 also reports the invalid state; malformed state cannot exempt a dependency |

`Node::is_complete_requirement` defines the completion boundary. `Store::decision_dependencies` is the shared projection of recorded edges into current or historical dependencies and their recorded superseding decisions. Lint, scheduling, display, and handover consume this projection. Neither role nor an acknowledgement flag is written back to the node.

`next` stops traversing work prerequisites when it reaches a completed requirement. Historical decisions and questions from finished work must not become new prerequisites of later work. This does not waive structural checks: missing targets, inverse-link inconsistencies, explicit unresolved references, and invalid completion records remain independently diagnosable.

Reopening a requirement changes its dependency role back to current without editing its edges. Compression preserves the edges and completion state, so it preserves their historical role. Updating a live product after earlier work has completed belongs to another active requirement, or to an explicitly reopened requirement; a completed task is not a declaration that its resulting behavior must remain in production forever.

This classification does not establish that a decision was valid at the time of completion, that an implementation changed, or that someone reviewed a supersession. The ledger cannot reconstruct unrecorded past events from today's state. Historical display is not a verification claim.

## Workflow policy over time

A project's current workflow configuration governs ongoing work and new actions. Introducing or changing a profile does not establish obligations for already completed work. Lint inspects completed requirements' recorded workflow snapshots using the schemas and comparisons saved with those snapshots. It does not demand new records from the current profile, including records marked `required`.

A new transition always evaluates the current destination guards, including reopening completed work and explicitly entering `complete` again. Explicit record submission also uses the current schema. This distinction is represented by separate passive inspection and current-action validation paths, rather than a per-node exemption flag, import marker, timestamp guess, or compression status.

Compression is an archival operation, not a work transition. It validates existing historical snapshots and core completion/archive requirements, then preserves the original data. Completed work without workflow snapshots can be archived without constructing a fictional history. Existing snapshots remain subject to their own recorded schemas and checks even if the current profile changes or is removed. Neither absence nor presence of a snapshot proves external approval or correct implementation.

## Rule contracts

| Consumer | Authoritative input | Result |
| --- | --- | --- |
| L1 and question-close warnings | `waiting-on` / `unresolved` references and the target question's `status` | An explicit unresolved reference conflicts with a closed question |
| L2 | Optional nonnegative integer `bearer_count` and needs' `targets` edges | A declared count must have the right type and equal the supporting-need count |
| L5 | Current requirement decision dependencies and recorded `supersedes` / `superseded-by` edges | Work still to be done depends on a replaced decision |
| Workflow lint on completed requirements | Saved workflow inputs, schemas, and comparisons | Check historical records without retroactive application of the current profile |
| New transitions and submissions | Current workflow configuration and supplied records | Validate the action now, regardless of the node's previous completion state |
| L10 / L11 / L13 / `edges` | State, completion records, references, and edge declarations | Structural integrity continues to apply to completed work |
| `show` / Markdown `render` | The dependency projection | Label superseded dependencies as current or historical without modifying canonical files |
| `handover` | The same projection and lint results | Separate `historical_superseded_dependencies` from actionable diagnostics |

Handover's historical entries and show's `decision_dependencies` contain `requirement`, `decision`, `role`, and `superseded_by`. They are derived output, not replacement frontmatter. Historical entries do not affect handover's exit code. Recorded successors are collected from both edge directions so a broken inverse link cannot silently erase supersession; the `edges` rule still reports that structural defect.

CLI and MCP invoke the same core operations. New checks should define their authoritative fields, relationship meaning, lifecycle applicability, and consuming views before implementation. Verification should test those contracts across consumers, including changes that must leave results invariant, rather than only a reported example.

## Compatibility and rejected approaches

L1 no longer interprets body keywords, and L2 no longer reads a prose `bearers 3` count. Users who intend those assertions must explicitly record `waiting-on`, `unresolved`, or `bearer_count`. No automated rewrite can reliably distinguish an assertion from a quote, code identifier, or negated sentence, so this change does not fabricate attributes during migration. Use `find` to review prose and write declarations based on evidence.

A word-boundary adjustment would retain two competing sources of authority. A separate heuristic review feature could be designed later with explicit provenance and separate results, but it does not belong in these integrity rules.

A per-node L5 acknowledgement would obscure whether a dependency is historical or current. Treating every completed node as exempt from all lint would instead hide broken history. The lifecycle projection avoids both: it preserves recorded relationships and applies each rule according to the meaning of its assertion.

The regression contract covers prose-edit invariance, structured assertions overriding prose, completion and reopening, scoped historical output, scheduling through completed work, and persistence of structural errors. Existing compression, workflow, and MCP tests verify the shared storage and execution paths.
