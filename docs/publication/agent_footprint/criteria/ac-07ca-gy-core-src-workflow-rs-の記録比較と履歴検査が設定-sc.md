# ac-07ca (AC-19) gy-core/src/workflow.rs の記録比較と履歴検査が設定 schema と別ファイルにあり、分割の前後で cargo test --workspace --locked の結果が同一である

- 種類: criterion
- scope: agent_footprint
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-19

## 関係

- targeted-by n-6dab (N-16) workflow.rs を関心ごとに分割し、テストを有効な基準台帳のフィクスチャから構成する

## 本文

## 測り方

0.4 系の受け入れ条件。対象のニーズは D-80 / d-736e で閉じた（消した機能か、新しい gy が別の形で満たした）。測り方は当時の satisfy の evidence に記録がある。

## 充足

- satisfied（N-16 / D-35. crates/gy-core/src/workflow.rs 8 lines; workflow/schema.rs 425, comparison.rs 107, history.rs 93, store.rs 167; existing declared_files.rs 264 unchanged. All four moved bodies equal d334001 after removing internal visibility qualifiers. lib.rs unchanged; downstream gy_core::RecordSchema import compiles (exit 0). cargo test --workspace --locked: before/after 68 passed, 0 failed, 0 ignored; all 68 names identical. Logs: /private/tmp/gy-n16-before-tests.log and /private/tmp/gy-n16-after-tests.log. cargo fmt --all -- --check exit 0; cargo clippy --workspace --all-targets --locked -- -D warnings exit 0.） 2026-09-13T12:03:48.833825+00:00

## 自由属性

- 無し

