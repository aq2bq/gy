# n-cda9 (N-23) 詳細本文の Markdown 描画を GFM の表・入れ子リスト・コード・リンクに対応させる

- 種類: need
- scope: html_projection
- created: 2026-09-13
- 状態: closed
- 別名: N-23

## 関係

- spawned-by d-47e7 (D-42) 本文は台帳の Markdown を欠けずに描く
- targets ac-a6d0 (AC-28) 詳細の本文で GFM の表・入れ子リスト・コードブロック・リンクが台帳本文どおりに描かれ、埋め込みエスケープの既存テストが通る

## 本文

## 要求（2026-09-13、マスター承認済み。開発担当 deck へ振り出し）

根拠は D-37（業務: 読む）と D-42。N-19〜N-21 とは独立。判定は AC-28。

### 満たす条件

1. 詳細の本文で、GFM の表（ヘッダー行、区切り行、揃え）、入れ子の箇条書きと番号付き、フェンス付きコードブロック、行内コード、リンク、強調が台帳本文どおりに描かれる。
2. 描画は `detail.js` の自作レンダラを拡張して行う。外部ライブラリの同梱が必要と判断したら、ライセンスとサイズを添えて question にする。
3. 埋め込みエスケープの契約（`<`、`>`、`&`、U+2028、U+2029）と、本文中の HTML をそのまま実行しないことを維持する。
4. E2E の読み物フィクスチャ（`e2e/fixtures.mjs` の reading fixture）に表と入れ子リストを含む本文を足し、描かれたセル数と階層を測る。

### 共通の維持条件と完了

- 派生表示の契約（architecture.md HTML projection 節: lint 等の判断を画面で計算しない、埋め込みエスケープ、バナーの件数表示）と Rust 側のテストを維持する。HTML の変化は互換な変更（z）に属する。
- 既存の E2E（`e2e/tests/`）は通す。D-39 により期待値が変わるテスト（クリック=焦点移動を前提にしたもの）は、その理由を D-39 として記して変更する。
- 検証は `e2e/` の Playwright テストで行い、受け入れ条件の測定値を evidence に書く。terminal-browser は目視用。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`e2e` の `npm test` が exit 0。
- need ごとにコミットし、push と公開はマスターの指示を待つ。報告は `herdr agent prompt kuroko` で D-33 の形。判断に迷う点、原則（D-38〜D-43）と衝突する点は question として台帳に置いて私へ送る。

## 完了確認（kuroko、2026-09-13）

deck の報告をコミット 0a17a70（N-23）と 159861c（N-21 追補: 再描画で入力途中の hopFrom が消える問題の修正）で照合。隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed / 0 failed、E2E 28 passed / 0 failed。外部依存の追加なし。受領。基盤 4 件（N-19〜N-21、N-23）が揃った。push・公開は未実施。

## 閉じ方

- 事実で閉じた（未実装のまま閉じた。HTML 投影は D-79 で消す (D-80)）

## 自由属性

- 無し

