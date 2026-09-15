# n-6dab (N-16) workflow.rs を関心ごとに分割し、テストを有効な基準台帳のフィクスチャから構成する

state: open
scope: agent_footprint
created: 2026-09-13
  spawned-by d-ecff (D-32) AGENTS.md は判断の所在と型だけを記述し、手順・ツールの癖・過去の失敗はツール、docs、台帳の decision に置く
  targets ac-07ca (AC-19) gy-core/src/workflow.rs の記録比較と履歴検査が設定 schema と別ファイルにあり、分割の前後で cargo test --workspace --locked の結果が同一である
  targets ac-7b51 (AC-20) 統合テストが有効な基準台帳を作る共通フィクスチャから始まり、handover と下流の実行例がそのフィクスチャから構成される
## 出所

2026-09-13、N-11 / N-13 の改修でデック先生のトークン消費が想定を超えたとマスターが報告し、デック先生の自己分析（ソース取得範囲の広さ、手戻り4件）を受けた。

## 観測

- `gy-core/src/workflow.rs` は 773 行 1 ファイルに、設定型・schema 検査・パス解決・値検査・記録比較（`string_set` / `check_records`）・履歴検査（`snapshots` / `history_issues`）・`Store` への impl が同居する。`declared_files.rs`（264 行）だけが分離されている。
- `gy/tests/workflows.rs` は 2150 行 37 テスト（平均 58 行）、`configured_workflow.rs` は 1083 行 15 テスト（平均 72 行）。テスト名に `_and_` を含むものが 36 本ある。
- フィクスチャは `add_ac` / `add_q` / `add_d` の単体ノード生成のみで、有効な基準台帳（担当者・次の根拠・created を備えた状態）を作る助けが無い。handover テストの必須項目不足と下流例の `created` 欠落は、この不在の症状である。

## 判定

採用する。読み込み範囲を関数単位に絞る手掛かりは物理構造に置く。進め方の注意書きでは代替しない（D-32 の適用範囲）。`declared_files.rs` の分離が前例である。

## 要求（2026-09-13、マスター承認済み。開発担当 deck へ振り出し）

根拠は上の観測。状態は 0.4.0 公開後の `main`（d334001）。判定は AC-19 と AC-20 の evidence で行う。

### 満たす条件

1. `crates/gy-core/src/workflow.rs` を関心ごとのサブモジュールに分ける。`workflow/declared_files.rs` が前例。区分の目安は、設定型と schema 検証（`WorkflowConfig` / `RecordSchema` / `FieldSchema` / `validate_*` / `schema_path`）、記録比較（`path_values` / `string_set` / `check_records`）、履歴検査（`snapshots` / `history_issues`）、`Store` への impl。1 ファイルの top-level の関心は 1 つ。
2. `gy_core` 直下の既存の公開 API（`gy_core::RecordSchema` など、`lib.rs` の `pub use workflow::*` が出す型・関数・enum のパスと形）を同一に保つ。`workflow` モジュールは非公開のまま内部で分割する。下流は `use gy_core::RecordSchema` のまま通る。（Q-28 → D-35 で訂正。改訂前は `gy_core::workflow::RecordSchema` と誤記していた）
3. 統合テストに、`gy lint` を通る有効な基準台帳（担当者・`next_evidence`・`created` を備えた requirement を含む）を作る共通フィクスチャを置く。handover のテストと、`docs/` の下流実行例（migration の下流クレート例）は、そのフィクスチャから構成する。
4. テストファイルはモジュール分割に対応させてよい。既存のテストの期待値は同一に保つ。

### 維持する条件

- 分割の前後で `cargo test --workspace --locked` の結果が同一（テスト件数と成否）。
- 保存形式、CLI・MCP の出力、診断と終了コード、`gy.toml` の解釈は無変更。互換な変更として `z` に属する。
- `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が exit 0。

### 完了

- AC-19 と AC-20 を `gy criterion satisfy --evidence` で閉じる。evidence は、分割後のファイル一覧と行数、前後のテスト件数、fmt / clippy の終了コード。
- 変更はコミットまで進め、push と公開はマスターの指示を待つ。コミットメッセージは英語で変更理由を書く。
- 報告は `herdr agent prompt kuroko` で、D-33 の形（台帳の ID、そこからの差分、測定結果、残った判断）で送る。

## 完了確認（kuroko、2026-09-13）

deck の報告をコミット 5cb1e0a で照合。workflow.rs 8 行 + schema.rs 425 / comparison.rs 107 / history.rs 93 / store.rs 167（declared_files.rs 264 は不変）。lib.rs は不変で公開パスは crate 直下のまま（D-35）。tests/support/mod.rs の baseline() は lint exit 0 を毎回確認する共通フィクスチャ、docs/migration-0.4.md の下流例も同じ fixture を読む。kuroko 側の再実行: fmt exit 0、clippy 警告なし、cargo test --workspace --locked 68 passed（15+7+37+9）/ 0 failed。AC-19・AC-20 satisfied。push・公開は未実施、マスター判断待ち。


