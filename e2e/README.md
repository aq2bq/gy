# E2E of the gy serve screen

With Chromium only, it checks the screen that `gy serve` serves against the API's JSON (d-244b).
The tests build the ledger in a temporary directory with the CLI. It is not part of `cargo test`.

```sh
cargo build --release -p gy
cd e2e && npm ci && npx playwright install chromium && npm test
```
