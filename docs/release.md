# Release procedure

This document is the source of truth for how a gy release is prepared,
verified, and published. Who decides what is defined in the local agent
operating rules; this page holds the steps and the checks.

## Decide the version

Compare the working tree with the most recently published version across the
Rust public API, the CLI and MCP surface, the on-disk ledger format, `gy.toml`,
diagnostics and exit codes, and how history is interpreted. Under `0.y.z`,
compatible fixes and additions bump `z` and incompatible changes bump `y`; from
`1.0.0`, patch, minor, and major follow semver. Test volume and change size do
not decide compatibility. Incompatible changes ship together in one version, and
when a public type is expected to gain fields later, the move to an extensible
shape such as `#[non_exhaustive]` ships in that same version.

Development builds keep the published version number until release. At release
time, align `gy` and `gy-core` versions, the CLI's dependency specification, and
`Cargo.lock` in one commit that is separate from feature commits. Commit
messages are in English and state the reason for the change.

## Prepare (reversible)

1. Write the English `CHANGELOG.md` entry and the migration guide. Any manual
   step left to existing users, including what to run against existing ledgers
   when a new behavior only applies at creation time, is written in both the
   changelog and the README. State explicitly when a command is safe to re-run.
2. Compile and run every code sample in the migration guide from a separate
   downstream crate. A sample that reads well and a sample that compiles are
   different things. Outside its defining crate, a `#[non_exhaustive]` struct
   rejects struct expressions and `..Default::default()` (E0639); downstream code
   takes `Default::default()` and assigns public fields. A `#[non_exhaustive]`
   type without `Default` is constructible downstream only through Deserialize;
   when that is intended, say so and why in the changelog.
3. Run `cargo fmt --all -- --check`,
   `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
   `cargo test --workspace --locked`.
4. Run `cargo publish --workspace --dry-run --locked`. Add `--allow-dirty` only
   for pre-commit verification; the publish itself runs from a clean, verified
   commit.
5. Run `cargo package --list` for each crate and confirm every file embedded with
   `include_str!` or similar is in the package. A missing file passes the local
   build and breaks only the published crate.
6. Commit the version alignment.

## Publish (irreversible)

A crates.io publish cannot be undone. `yank` only stops new dependents from
selecting the version; the number stays consumed and `--locked` users keep
resolving it. Publish after the reversible steps are complete and the publish
has been authorized.

1. `cargo publish -p gy-core --locked`, then `cargo publish -p gy --locked`.
2. Push the release commit and the matching `vX.Y.Z` tag.
3. Install from the registry and run the binary once:
   `cargo install gy --locked && gy --version`.
4. Tell users the published version and the update command. `--locked`
   reproduces the bundled dependency versions.

Published versions are immutable artifacts; corrections ship in the next
appropriate version.
