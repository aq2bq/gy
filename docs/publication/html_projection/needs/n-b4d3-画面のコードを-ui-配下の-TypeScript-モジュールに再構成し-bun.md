# n-b4d3 (N-25) 画面のコードを ui/ 配下の TypeScript モジュールに再構成し、bun で束ねた成果物を html.rs が埋め込む形にする

state: closed
scope: html_projection
created: 2026-09-13
  spawned-by d-a2c6 (D-45) 画面のコードは関心ごとの ES モジュールと TypeScript で書き、bun で 1 本に束ねた成果物を html.rs が埋め込む
  targets ac-d906 (AC-30) 画面のソースが ui/ 配下の関心ごとのモジュールにあり、グローバル変数の共有が無く、bun build の成果物がコミット済みで CI が再現一致を検査し、再構成の前後で e2e の全テストが通る
  depended-on-by n-1b72 (N-27) 押せる見た目と押せる実体を一致させ、Overview の数値カードを一覧への入口にする
  depended-on-by n-3f2d (N-26) ツールバーと状態表示を設計トークンに基づいて組み直し、絞り込み・グラフ操作・リセットを分ける
  depended-on-by n-4759 (N-24) タイトルを一覧と詳細で全文、グラフのラベルで折り返し表示にする
  depended-on-by n-ca04 (N-22) 絞り込み結果の一覧ビューを置き、クラスタと種別のクリックを一覧へ導く
## 出所

2026-09-13、マスターの目視と指摘（N-26 / N-27 の出所と同じ）。画面の JS は `crates/gy-core/src/html/` の 19 ファイルを `html.rs` が `include_str!` で連結し、グローバル変数（`focusHistory`、`selected`、`scale` など）を全ファイルで共有している。関心の境界が無いので配置の原則も書けず、workflow.rs と同じ物理設計不足が起きている。

## 要求（2026-09-13、マスター承認済み。開発担当 deck へ振り出し）

根拠は D-45。状態は `main` 0a17a70。判定は AC-30。これは振る舞いを変えない再構成であり、E2E の 28 件が契約である。

### 満たす条件

1. **ソースの置き場**: 画面のソースを `crates/gy-core/ui/` に置く。`package.json`、`tsconfig.json`、`src/` を含む。bun 1.4 系を使い、`bun install` と `bun run build` で成果物を作る。依存を足す場合はライセンスとサイズを添えて question にする（まずは依存なしで再構成する）。
2. **モジュール境界**: 最低限、次の関心を別モジュールにする。`state`（表示状態の型と、hash codec の唯一の定義、状態変更の関数）、`data`（埋め込み payload の読み出しと索引）、`filters`（検索・絞り込み）、`list`（一覧、N-22 の置き場）、`detail`（詳細と Markdown レンダラ）、`graph`（layout / svg / draw / viewport / clusters）、`survey`（Overview / Progress / Blockers）、`tokens`（余白・文字寸法・色の設計トークンを CSS 変数として 1 か所に）。モジュール間の共有はグローバル変数ではなく import で行い、状態は `state` からしか変えない。
3. **成果物と埋め込み**: `bun run build` が `crates/gy-core/src/html/dist/app.js` と `app.css` を出す（minify なし、決定的な出力）。`html.rs` は head / body / tail のテンプレートと dist の 2 ファイルを埋め込む。`cargo build` と `cargo package` は bun を要求しない。`cargo package --list` で dist が配布物に入ることを確認する。
4. **再現検査**: `ui/` に `bun run check`（build して `git diff --exit-code` で dist の一致を確かめる）を置き、`.github/workflows/ci.yml` の HTML ジョブに bun のセットアップと `bun run check` を足す。
5. **振る舞いの不変**: `e2e/` の 28 件が変更なしで通る。Rust の `html_render` テストが通る。出力は単一 HTML でネットワークへ出ない（既存の E2E が検査する）。
6. **文書**: `docs/architecture.md` の HTML source assembly 節を新しい組み立てに同期し、`ui/README.md` に build / check の手順と、dist をコミットする理由（cargo build が bun を要求しないため）を書く。

### 維持する条件

- 埋め込みエスケープの契約と payload の形は不変。
- 互換な変更（z）。CLI・MCP・保存形式に触れない。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`e2e` の `npm test`、`bun run check` が exit 0。

### 完了

- AC-30 を evidence 付きで satisfy。evidence は、モジュール一覧と行数、グローバル共有が無いことの確認方法（例: `window` への代入を grep して 0 件、または TypeScript の `noImplicitAny` と `isolatedModules` が通る）、dist のサイズ、E2E 28 件の結果、CI の check ジョブの結果。
- コミットは「bun と ui/ の導入」「モジュール分割」「html.rs の埋め込み変更」のように理由ごとに分けてよい。push と公開はマスターの指示を待つ。
- 報告は `herdr agent prompt kuroko` で D-33 の形。判断に迷う点、D-45 と衝突する点は question として台帳に置いて私へ送る。

## 完了確認（kuroko、2026-09-13）

deck の報告をコミット f47b0c8 で照合。隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed、bun run check exit 0、E2E 28 passed / 0 failed。ui/src は 22 モジュール（state / data / filters / list / detail / graph / survey / tokens / templates）。window への代入は `templates/body.html` の `window.GY_DATA = __PAYLOAD__` の 1 件だけで、これは Rust が埋め込む payload の受け渡しであり状態ではない。cargo package --list に dist 2 ファイルと templates 3 ファイル、ui/ のソースが入る。Q-30 は D-46（ローカル同一コマンドで受け入れ、push 後に GitHub の結果を追補）で閉じ、AC-30 satisfied。受領。

閉じた理由: 事実（migrated: complete）

