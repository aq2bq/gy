# n-ac8e (N-21) 検索と絞り込みをヘッダー直下の常設ツールバーにし、左パネルを状況把握に限る

- 種類: need
- scope: html_projection
- created: 2026-09-13
- 状態: closed
- 別名: N-21

## 関係

- spawned-by d-3026 (D-41) 検索と絞り込みは常設のツールバーとしてヘッダー直下に置く
- targets ac-b77a (AC-26) 検索欄と種別・スコープ・状態の絞り込みがヘッダー直下に常に見え、左パネルは Overview / Blockers / Progress だけになる

## 本文

## 要求（2026-09-13、マスター承認済み。開発担当 deck へ振り出し）

根拠は D-37（業務: 絞り込んで一覧を読む・状況把握）と D-41。状態は N-20 の後。判定は AC-26。

### 満たす条件

1. 検索欄、種別チップ、スコープ、状態、リセットをヘッダー直下の常設ツールバーに置く。タブ切替なしに常に見える。
2. 左パネルは Overview / Blockers / Progress の 3 つになる。Filter タブは無くなる。Progress や Blockers から絞り込みを設定する導線（`setStateFilter`）はツールバーの状態を変えるだけで、タブ移動を伴わない。
3. genealogy の切り替えと近傍の半径指定は、状態が見える形でツールバーか詳細パネルに置く（既存のモードは維持し、新しいモードは足さない）。
4. 1280 / 1440 / 1920 の幅でツールバーが 1 行に収まるか、折り返しても操作が隠れない。

### 共通の維持条件と完了

- 派生表示の契約（architecture.md HTML projection 節: lint 等の判断を画面で計算しない、埋め込みエスケープ、バナーの件数表示）と Rust 側のテストを維持する。HTML の変化は互換な変更（z）に属する。
- 既存の E2E（`e2e/tests/`）は通す。D-39 により期待値が変わるテスト（クリック=焦点移動を前提にしたもの）は、その理由を D-39 として記して変更する。
- 検証は `e2e/` の Playwright テストで行い、受け入れ条件の測定値を evidence に書く。terminal-browser は目視用。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`e2e` の `npm test` が exit 0。
- need ごとにコミットし、push と公開はマスターの指示を待つ。報告は `herdr agent prompt kuroko` で D-33 の形。判断に迷う点、原則（D-38〜D-43）と衝突する点は question として台帳に置いて私へ送る。

## 完了確認（kuroko、2026-09-13）

deck の報告をコミット 4163149 で照合。隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed / 0 failed、E2E 26 passed / 0 failed。受領。push・公開は未実施。

## 閉じ方

- 事実で閉じた（未実装のまま閉じた。HTML 投影は D-79 で消す (D-80)）

## 自由属性

- 無し

