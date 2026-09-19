# The release procedure

This document defines how gy's version is decided, how a release is prepared, and how it is published. Who judges what is in `AGENTS.md`; this document holds the procedure and the items to check.

## Decide the version

Compare the working tree with the last published version on each of: the public Rust API, the surface of the CLI, the storage format, `gy.toml`, the diagnostics and exit codes, and the interpretation of the history. In `0.y.z` a compatible change moves `z` and an incompatible change moves `y`. From `1.0.0` on, follow semver. Do not decide by the amount of tests or the size of the diff. Put incompatible changes together in one version.

A build under development keeps the last version number until the release. At release time, align the versions of `gy-ledger`, `gy-serve` and `gy`, the version requirements of the crates the CLI depends on, and `Cargo.lock` in one commit. Keep this commit separate from feature commits. The commit message is in English and states the reason for the change.

## Prepare (reversible)

1. Write the English `CHANGELOG.md` and the migration guide (`docs/migration-0.5.md`). Manual work left to the user is written in both the CHANGELOG and the README. Also state whether it is safe to run the same command again.
   At the same time, check `docs/architecture.md`, the README in English and Japanese, `crates/gy/skills/gy-loop/CHEATSHEET.md` and the bundled skills (`crates/gy/skills/*/SKILL.md`) against each entry of the CHANGELOG, and confirm that changes to output and contracts are reflected in the documents and the skills. The skills are bundled in the crate, so fix them before the release (this was missed in 0.9.0; d-f7b6). For a version that changes the skills, write in the CHANGELOG's Updating section that "the skills changed, so copy them again", with a reference to the README's section on where to put the skills (n-f8a2).
2. Run the code examples in the migration guide from a downstream crate and confirm them. An example that reads well and an example that compiles are different things. A `#[non_exhaustive]` type does not accept a struct literal or `..Default::default()` outside the crate that defines it. Downstream creates `Default::default()` and assigns to the public fields.
3. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --locked`.
4. Run `cargo publish --workspace --dry-run --locked`. It packages and verifies `gy-ledger`, `gy-serve` and `gy` in the order of dependency. For `gy-serve`, confirm with `cargo package --list -p gy-serve` that every file under `src/assets/` embedded with `include_str!` is in the package (since 0.6.1). Use `--allow-dirty` only for checks before the commit, and publish from a clean, verified commit.

> When the dry-run is repeated with the same version number, the gy-ledger artifacts built in the previous verification (`target/` and `~/.cargo/registry/src/*/gy-ledger-<version>`) are reused, and the verification of `gy` can fail against an old API. In that case, run `cargo clean`, delete that directory, and do the dry-run again (confirmed on 2026-09-15 while preparing 0.5.0).

5. Run `cargo package --list` for each crate and confirm that the embedded files are included in the package. A missing file passes the local build and breaks only in the published crate.
6. Make the commit that aligns the versions.

## Publish (irreversible)

Publishing to crates.io cannot be undone. `yank` only stops new dependencies from choosing that version; the number stays consumed. Publish after the reversible preparation is finished and the instruction to publish has been given.

1. `cargo publish -p gy-ledger --locked`, then `cargo publish -p gy-serve --locked`, and last `cargo publish -p gy --locked` (since 0.6.1 `gy-serve` comes in between: `gy` depends on `gy-serve`, and `gy-serve` depends on `gy-ledger`).
2. Push the release commit and the matching `vX.Y.Z` tag.
3. Install from the registry and run the binary once: `cargo install gy --locked && gy --version`.
4. Tell the users the published version and the command to update. `--locked` reproduces the versions of the bundled dependencies.

A published version is an immutable artifact. A fix is delivered in the next appropriate version.
