# E2E of the gy serve screen

With Chromium only, it checks the screen that `gy serve` serves against the API's JSON (d-244b).
Browser tests use isolated temporary ledgers: small scenarios use the CLI, and the load fixture uses gated Rust transactions. They are not part of `cargo test`.

```sh
cargo build --release -p gy --bin gy --example large_fixture --locked
cd e2e
./without-node.sh bun install --frozen-lockfile
./without-node.sh bun run test:unit
./without-node.sh bunx --bun playwright install --with-deps chromium
./without-node.sh bunx --bun playwright test
```

`./without-node.sh bun run test` runs the JavaScript structure checks before the browser tests.
`./without-node.sh bun run test:unit` runs those checks alone, without a browser or a Rust build.
The unit script limits Bun's test collection to `unit/`; Playwright runs the browser specs.
The wrapper excludes Node.js command names from the check's PATH, verifies that
`node` and `nodejs` cannot be found, and prints the actual Bun version and revision.
CI follows Bun's latest stable release; Playwright and yuku are pinned by `bun.lock`.
They parse syntax and resolve variable bindings, ignoring comments and strings.
In a region or part, assignments to `state` or its properties are refused,
including local aliases of supplied state. A local declaration alone is allowed.

Small fixtures exercise CLI writes. The large fixture uses the Rust model and
normal gated transactions to create exactly 2,170 nodes without thousands of
CLI process launches. Its scopes contain 900, 901 and 369 nodes. Tests assert
these counts and use filtered requests to exercise the rendering threshold;
an unfiltered request always returns a grid, regardless of size. This is a
synthetic load fixture, not a copy of a user's ledger or its history.

The load fixture has 607 needs, 685 criteria, 503 decisions, 173 questions and
202 requirements, with creation dates spanning 31 days. These approximate gy's
observed kind proportions; they do not reproduce its relationships or write
history. Tests assert the proportions, date spread and 607 overview cards.
