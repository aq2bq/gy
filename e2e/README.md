# E2E of the gy serve screen

With Chromium only, it checks the screen that `gy serve` serves against the API's JSON (d-244b).
The tests build the ledger in a temporary directory with the CLI. It is not part of `cargo test`.

```sh
cargo build --release -p gy --bin gy --example large_fixture --locked
cd e2e && npm ci && npx playwright install chromium && npm test
```

`npm test` runs the JavaScript structure checks before the browser tests.
`npm run test:unit` runs those checks alone, without a browser or a Rust build.
They parse syntax and resolve variable bindings, so comments, strings and local
variables cannot be mistaken for mutations of the shared state.

Small fixtures exercise CLI writes. The large fixture uses the Rust model and
normal gated transactions to create exactly 2,170 nodes without thousands of
CLI process launches. Its scopes contain 900, 901 and 369 nodes. Tests assert
these counts and use filtered requests to exercise the rendering threshold;
an unfiltered request always returns a grid, regardless of size. This is a
synthetic load fixture, not a copy of a user's ledger or its history.
