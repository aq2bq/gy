# Changelog

## 0.4.2

### Added

- A `[workflow.guards.<name>]` entry may set `waived_by` to the name of a
  project-defined record whose schema applies to requirements. A requirement
  carrying that record validly is not asked for the guard's `records` or
  `checks`. The waiver is preserved in the transition snapshot as a `waived`
  map, and checks from waived guards are not stored. An absent record leaves
  the guard in force, and an invalid one is reported while the guard still
  applies. `gy handover --json` includes `waived_by` in `workflow.guards`.
  A project that does not configure `waived_by` is unaffected.

## 0.4.1

### Added

- `gy import` marks every decision node it creates with the node attribute
  `imported: true`, whether or not `decision_scope` was filled. An imported
  decision whose applicability conditions are empty is reported as L14
  (default `warn`) instead of L7, so a legacy ledger taken in with
  intentionally empty scopes no longer fails `gy lint` and `gy handover`.
  L14 is configured with `[lint] L14` like any other rule. Decisions imported
  before this release carry no mark; collect their IDs from L7 findings in
  `gy lint --json` and run `gy node set <ID> --set imported=true` to bring
  them under the rule. Setting `decision_scope` clears the finding.

## 0.4.0

### Breaking

- Mutation results now return node summaries (`id`, `type`, `scope`,
  `changed_attributes`, `body_changed`) instead of full nodes. Attribute names
  describe actual value changes, including removals; creation lists all names.
  Repeating an unchanged update returns an empty list and `false`.
  This applies to CLI and MCP, including human output. Existing `node`,
  `need`/`requirement`, and `source`/`target` containers now hold summaries;
  import's `imported` array holds summaries and scope rename returns `nodes`.
  Warnings and import review information remain available. Submit returns
  `submission.record` and `submission.revision` (null without a version field),
  not the submitted records, schemas, or evidence. Init retains `root` and `scope`.
- Migrate consumers of mutation responses to `gy show <ID> --json` for all
  attributes and body text, or `gy find` for search. Compression with evidence
  also returns a summary without `archive`; obtain the full archival text with
  `gy req compress <issue>` before supplying evidence. That read-only preview
  and other query results are unchanged.

- Add `CheckKind::MatchesDeclaredFiles` and mark `DependencyRole`, `RuleConfig`,
  `FieldType`, and `CheckKind` as `#[non_exhaustive]`. Downstream matches must
  include a wildcard arm. Known enum variants remain constructible, including
  `RuleConfig::Detail { enabled: Some(true), severity: None }`. This differs
  from the non-exhaustive structs introduced in 0.3, whose downstream struct
  literals remain forbidden. See the [compiled migration example](docs/migration-0.4.md#update-rust-clients).
- The shipped workflow example now declares design files as `literal` or
  `generated` objects and compares them with `matches-declared-files`.
  Existing profiles keep their current behavior. To adopt the new shape,
  replace the `design_proposal.fields.files` schema using the
  [minimal profile](crates/gy/examples/file-scope.toml), change the file guard
  to `matches-declared-files` with implementation files on the left and design
  declarations on the right, and remove comparison keys or `[]` projections.
  Convert fixed design paths to `{ "kind": "literal", "path": "..." }` and
  generated names to fixed directory/prefix/suffix plus token class and length.
  Keep implementation paths as concrete strings. Already recorded designs need
  a new revision and matching approval; do not rewrite old snapshots. See the
  [complete migration steps](docs/migration-0.4.md#update-workflow-profiles-and-records).

### Added

- Restore HTML reading state from the URL hash, including filters, the selected
  record, tab, focus path, and view controls. Browser Back/Forward and reload
  restore the corresponding view; missing IDs are reported explicitly.
- Keep a permanent toolbar and sortable record table available while reading
  graph neighborhoods and details. Overview counts link to their corresponding
  record sets, derived from the HTML's embedded ledger data.
- Render Markdown tables, nested lists, fenced code, emphasis, and safe links
  in record bodies while retaining the offline, self-contained HTML boundary.
- Provide fixture caching keyed by CLI binary and generator SHA-256 hashes,
  explicit forced regeneration, and `test:spec`, `test:all`, and `test:perf`
  E2E entry points.

- Optional `description` strings on workflow records and fields appear in
  handover JSON and saved schemas without affecting checks or record revisions.
  Existing data needs no migration. Upgrade every ledger reader before using
  descriptions: older binaries reject configurations and histories containing
  them. Removing descriptions from today's configuration does not remove them
  from saved history. Older binaries also reject the new file comparison.
- The opt-in file comparison permits one generated filename token with an exact
  ASCII digit or lowercase hexadecimal length. Each declaration must match one
  reported file and each file one declaration. Undeclared, missing, duplicate,
  and ambiguous entries fail. No glob interpretation or file lookup occurs;
  existing `equal`, `same-set`, and `subset` semantics remain unchanged.

### Changed

- Record links open details; neighborhood navigation remains an explicit
  action. Cluster links filter the permanent table. Selected type chips,
  Lineage on/off, and a single status strip make the active view visible.
- Split the private workflow implementation into internal modules without
  changing existing `gy_core` public paths or type shapes.
- Organize HTML source as TypeScript modules and templates under `ui/`.
  Commit and package the Bun-built `app.js` and `app.css` embedded by Rust;
  end users do not need Bun, Node.js, or a browser automation installation.
- Run local `npm test` as 42 parallel functional checks. Use
  `npm run test:all` for the complete 45-check release/completion gate, or
  `npm run test:perf` for the three unchanged serial performance checks.
  CI continues to run all checks with one worker and zero retries.

### Fixed

- Align pointer cursors and visible controls with native links or registered
  button actions, including actual mouse hit targets across every table column.
- Wrap graph titles below IDs for up to three lines and include their measured
  bounds when fitting ordinary individual views. Preserve readable Lineage
  generation layout and panning. Full titles remain available in the table
  and details.
- Place individual edge labels away from node labels, omitting them when no
  nearby placement fits. All relationships remain available in node details;
  no edge-selection mode is introduced.
- Wait for URL persistence before history traversal in browser tests, then
  verify the destination URL, displayed tab, and unchanged history length.

- The example workflow accepts successful gates without a population through
  `passed-without-population`, requiring a reason and execution evidence.
  `passed` still requires its population and integer denominator. Existing
  users must add `passed-without-population` with required string `reason`
  and `evidence` fields to
  `workflow.records.quality_gates.fields.results.items.fields.result.variants`
  in their ledger configuration, preserving `passed`, local guards and past
  records. Use it only when no population concept exists; see the
  [workflow migration guide](docs/workflows.md#quality-gates-with-and-without-a-population).

### Migration

- Consumers that previously read a full node from a mutation response must use
  the returned ID to query `gy show <ID> --json`; read attributes and body from
  that query's `node`. See the Breaking section above for affected containers.
  This applies equally to CLI and MCP clients. For workflow profiles and saved
  records, follow the 0.4 migration steps described above; upgrade all readers before using the new
  schema descriptions or file comparison.
- Regenerate existing projections with `gy render --format html` to obtain the
  new UI. It is safe to repeat: rendering replaces derived HTML and does not
  change ledger records. Previously exported HTML retains its embedded UI.
- Contributors must use `npm run test:all` for completion evidence; local
  `npm test` intentionally omits performance checks. After editing `ui/`, run
  its `bun run build` and include the updated bundle in the change. See
  [the UI guide](crates/gy-core/ui/README.md) and
  [E2E commands](e2e/README.md).

## 0.3.1

### Changed

- Give details the same width as the graph on wide screens. Separate applicability
  and body from labeled declarations, show finite metadata as badges, and retain
  complete additional attributes. Relationship marks keep exact body annotations
  or explicit missing-location notices. Add browser checks for width, actual
  Japanese/Latin line wrapping, metadata, and lossless attributes.

- Focused neighborhoods remain individual even above 60 candidates. The graph
  selects up to 60 by distance, degree, and ID, with an omitted-node list for
  reaching the remainder. Omission and off-screen counts have separate actions.
- Add development-only Playwright checks for Chromium,
  including synthetic high-degree and 1000-node fixtures and automatic known-
  defect injection on every CI run. Node.js and browser binaries are not needed
  to build, install, or use gy.

- Clicking an individual graph node now moves the focus and opens details in a
  separate pane. Back, the focus path, and All nodes return through stored
  node/radius history without clearing filters, search, or lineage. A hidden
  focus remains in the path with an explicit notice. Reset everything clears
  navigation and filters together. Isolated nodes explicitly show that this
  graph contains no connections for the focus.
- Graph resizing preserves manual zoom and the graph point at the viewport
  center, while automatic views refit. Focus/displayed-node changes refit once
  after updating the detail pane. Automatic individual zoom is capped at 2
  to avoid over-enlarging isolated nodes; manual zoom still reaches 4. When a
  readable view cannot fit all selected nodes, it centers the focus (or the
  selected bounds if the focus is absent/filtered out) and reports off-screen
  nodes, including in lineage views.


- Split the HTML projection sources by responsibility and assemble them at
  compile time, preserving the generated HTML byte for byte. HTML generation
  still requires only the gy binary.

- The HTML graph now switches between two views by the number of visible
  nodes instead of by zoom level. Without a focus, more than 60 nodes form clusters
  with aggregated edge counts (per cluster pair) and internal edge counts per
  cluster, without individual nodes or edges. With 60 or fewer it draws
  individual nodes laid out by a force-directed layout that reflects
  connectivity, so an edge's length matches its endpoints' proximity.
- Clicking a cluster opens a member list (sorted by degree, title, or id,
  with a "current" mark for decisions not superseded); picking a member starts
  an adaptive neighborhood. The hop count is chosen per start as the largest
  value up to 5 whose reachable set fits the individual-view limit, and the
  chosen value is shown on screen.
- Node labels are measured with `getComputedTextLength`, truncated to the
  available width, and skipped when they would overlap a neighbour; the full
  title remains readable in the detail panel. Edge labels are off by default
  and drawn only when the individual set is small enough (20) or an edge is
  explicitly selected.
- Changing what is displayed refits the view, and the fit keeps individual
  labels at a legible size rather than shrinking them to fit every node; the
  count banner reports what is drawn in either view.
- The decision genealogy keeps its generation-layered layout.

### Upgrade

`cargo install gy --version 0.3.1 --locked`. No ledger migration is required
and the on-disk format and public API are unchanged. Regenerate existing HTML
with `gy render --format html` to use the new navigation and detail layout.

## 0.3.0

### Added

- `gy render --format html` projects the whole ledger into one self-contained,
  offline HTML file. It embeds the ledger's data and core judgments (lint, next,
  handover facts, stats, decision dependencies), draws all six node types and
  twelve relationship labels with direction, filters/scans the ledger, shows
  per-node details, and renders the decision lineage by generation.
- New `RenderConfig::html_output` (`gy.toml [render]`), defaulting to
  `gy.html` at the ledger root. Writing `{scope}` in it splits the output per
  scope. `split_threshold` never applies to HTML.
- `gy init` appends the default HTML output to the ledger root's `.gitignore`.
- Lint findings do not change `render`'s exit code: a broken ledger still
  renders so its problems can be seen from the projection.

### Changed

- **Breaking:** the public config structs `Config`, `ImportConfig`,
  `ScopeConfig`, `RenderConfig`, `WorkflowConfig`, `RecordSchema`,
  `FieldSchema`, `StateGuard`, and `RecordCheck` are now `#[non_exhaustive]`.
  Downstream crates can no longer build them with struct expressions of any
  form (exhaustive or functional update), because `#[non_exhaustive]` forbids
  struct expressions outside the defining crate. The new
  `RenderConfig::html_output` field is one reason; future render settings can
  now be added without another breaking release.
- **Breaking:** `RenderConfig` grows the `html_output` field (see Added). Rust
  library users constructing it exhaustively must update their code.
- The `--format html` value is accepted by `render` and `gy_render`.

### Upgrade

Install or update the CLI with `cargo install gy --version 0.3.0 --locked`.
No ledger migration is required: existing `gy.toml` files continue to parse
and the on-disk data format is unchanged.

#### Rust library users (`gy-core`)

Six config structs derive `Default`: `Config`, `ImportConfig`, `ScopeConfig`,
`RenderConfig`, `WorkflowConfig`, and `StateGuard`. For these, construct with
`Default::default()` and assign the public fields you need:

```rust
let mut render = gy_core::RenderConfig::default();
render.html_output = "gy.html".into();
```

The other three — `RecordSchema`, `FieldSchema`, and `RecordCheck` — have no
meaningful defaults and do not implement `Default`. They are read-only
configuration types: construct them by deserializing a `gy.toml`/JSON value
via `serde`. `RecordSchema::default()` and `FieldSchema::default()` do not
exist; this is deliberate, not a gap to fill in downstream code.

#### Existing ledgers: ignoring the HTML output

`gy init` appends `html_output` (default `gy.html`) to the ledger root's
`.gitignore`. When `html_output` contains `{scope}` it appends the expanded
pattern. For an existing ledger, either run `gy init <existing-scope>` again
(append-only, idempotent; nodes, relationships, records, and history are
preserved), or add the line by hand.

Re-running `init` also rewrites `gy.toml` in normalized form. Attribute values
are preserved, but comments and formatting are rewritten: inline table entries
expand to `[table]` sections, and the `html_output`, `[import]`, and
`[workflow]` defaults may appear if absent. If you keep operating notes as
comments in `gy.toml`, add the `.gitignore` line by hand instead of re-running
`init`.

## 0.2.1

### Added

- `gy scope rename <old> <new>` relabels a scope across its directory, member
  `scope` attributes, and `[scopes]` configuration without changing node IDs,
  relationships, records, or history. MCP exposes the same operation as
  `gy_scope`. CLI and MCP share the core operation; an existing destination is
  rejected rather than merged.

### Upgrade

Install or update the CLI with `cargo install gy --version 0.2.1 --locked`.
No ledger migration is required: the on-disk format and existing journal
entries are unchanged. Rust library users can set `gy-core = "0.2.1"`.

## 0.2.0

### Added

- Configurable workflow records, field types, explicit exceptions, and state
  guards, with matching CLI and MCP behavior.
- Stateless record submission and snapshots of validated records, schemas, and
  comparisons. Version checks prevent reuse of changed designs and approvals.
- Contract/gate coverage and declared file-scope comparisons.
- Workflow examples and shared authority/lifecycle architecture documentation.
- Historical superseded dependencies in show and handover output.

### Changed

- **Breaking:** `gy-core::Config` has new import and workflow fields. Callers
  using exhaustive struct literals must initialize them or use `Default`.
- **Breaking:** L1 uses explicit `waiting-on` / `unresolved` attributes; L2 uses
  the optional nonnegative integer `bearer_count`. Body text no longer declares
  unresolved references or counts. Record intended assertions in frontmatter.
- L5 checks current dependencies of unfinished requirements. Completed work
  retains its historical edges; reopening restores current checks.
- Current workflow policy governs ongoing work and new actions. Completed
  snapshots are validated against their saved schemas and comparisons, so
  adopting a profile does not require reconstructing past approvals.
- Scheduling stops following work prerequisites through completed requirements.

### Fixed

- ADR import can map a configured body section to `decision_scope` and reports
  missing scope and relationship marks. Imported missing marks remain visible
  as migration gaps.
- Multiple `raised-by` references are allowed; L4 checks `belongs-to` ownership.
- Code identifiers such as `waiting_checkout?` no longer cause L1 findings.
- Completed records can be compressed after profile adoption without fabricating
  historical workflow records. Existing completion/archive checks still apply.

### Upgrade

Install or update the CLI with `cargo install gy --version 0.2.0 --locked`.
Rust library users should set `gy-core = "0.2"` and update `Config` construction.
See `docs/architecture.md` for semantics and `docs/workflows.md` for adoption.
