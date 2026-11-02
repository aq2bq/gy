# Making and checking a change

Describe the user-visible change before editing. Identify the existing screens,
commands, and API responses it affects, and the tests that must still pass.
Review deleted or weakened tests separately from the implementation: explain
which requirement authorizes removing the behavior. The PR template records
this comparison; it does not require another approval for routine changes.

Run the focused checks while editing. For UI structure, `npm ci --prefix e2e`
and `npm run test:unit --prefix e2e` parse the JavaScript without building Rust
or starting a browser. Rust tests check the embedded asset inventory against
the same role table. For Rust behavior, select the affected package and test.

Before completion, run formatting, clippy, the workspace tests, and the UI tests
as defined in `.github/workflows/ci.yml`. The required `test` check aggregates
Rust, cargo-shear, and knots; `e2e` includes JavaScript unit checks and browser
tests. An API change and its UI consumer must pass together on the same commit.
Release packaging and publication remain in `release.md`.

The UI tests and the 2,170-node load fixture are described in `../e2e/README.md`.
Keep assertions about the fixture's actual size and both sides of a rendering
threshold. A test's title alone is not evidence that it exercised that size.

When changing bundled skills, compare the installed copy used by the agent
with `crates/gy/skills/`. Copy the changed skill again as described in the README.
The installed copy is outside the repository and cannot be updated by CI.

For the next few small changes, report time spent investigating, implementing,
reviewing, testing, and waiting, plus the number and reasons for rework rounds.
Keep those measurements in the PR or session report, not in the product ledger.
Use them to decide whether the next improvement belongs in code, tests, or
agent handoffs.
