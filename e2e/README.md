# gy serve の画面の E2E

Chromium だけで、`gy serve` が配る画面と API の JSON を突き合わせる（d-244b）。
台帳はテストが一時ディレクトリに CLI で作る。`cargo test` には入らない。

```sh
cargo build --release -p gy
cd e2e && npm ci && npx playwright install chromium && npm test
```
