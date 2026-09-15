# ac-7b51 (AC-20) 統合テストが有効な基準台帳を作る共通フィクスチャから始まり、handover と下流の実行例がそのフィクスチャから構成される

- 種類: criterion
- scope: agent_footprint
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-20

## 関係

- targeted-by n-6dab (N-16) workflow.rs を関心ごとに分割し、テストを有効な基準台帳のフィクスチャから構成する

## 本文

## 測り方

0.4 系の受け入れ条件。対象のニーズは D-80 / d-736e で閉じた（消した機能か、新しい gy が別の形で満たした）。測り方は当時の satisfy の evidence に記録がある。

## 充足

- satisfied（N-16 / D-35. Shared baseline: crates/gy/tests/support/mod.rs 28 lines and fixtures/requirement.md 10 lines. Includes created/responsible/next_evidence; helper runs gy lint --json and asserts exit 0 before each scenario. configured_workflow suite (15 tests) and both named handover tests pass using this fixture; missing handover fields are explicitly removed for the negative case. docs/migration-0.4.md, Update Rust clients: code now reads this same fixture (byte-identical downstream copy), asserts baseline core lint has 0 diagnostics. Extracted Rust block compiled and ran in downstream crate exit 0, covering enum matches, config loading, matching report acceptance and extra-file rejection; log /private/tmp/gy-n16-downstream.log. cargo test --workspace --locked: before/after 68 passed, 0 failed, 0 ignored; all 68 names identical. Logs: /private/tmp/gy-n16-before-tests.log and /private/tmp/gy-n16-after-tests.log. cargo fmt --all -- --check exit 0; cargo clippy --workspace --all-targets --locked -- -D warnings exit 0.） 2026-09-13T12:03:48.886334+00:00

## 自由属性

- 無し

