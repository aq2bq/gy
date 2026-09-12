# Changelog

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
