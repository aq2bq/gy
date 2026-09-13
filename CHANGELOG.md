# Changelog

## 0.3.1

### Changed

- Split the HTML projection sources by responsibility and assemble them at
  compile time, preserving the generated HTML byte for byte. HTML generation
  still requires only the gy binary.

- The HTML graph now switches between two views by the number of visible
  nodes instead of by zoom level. With more than 60 nodes it draws clusters
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
and the on-disk format and public API are unchanged.

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
