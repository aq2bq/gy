# n-3f2d (N-26) ツールバーと状態表示を設計トークンに基づいて組み直し、絞り込み・グラフ操作・リセットを分ける

- 種類: need
- scope: html_projection
- created: 2026-09-13
- 状態: closed
- 別名: N-26

## 関係

- depends-on n-b4d3 (N-25) 画面のコードを ui/ 配下の TypeScript モジュールに再構成し、bun で束ねた成果物を html.rs が埋め込む形にする
- spawned-by d-a2c6 (D-45) 画面のコードは関心ごとの ES モジュールと TypeScript で書き、bun で 1 本に束ねた成果物を html.rs が埋め込む
- targets ac-0b0b (AC-31) ツールバーが絞り込み・グラフ操作・リセットの3群に分かれて 1440px で 1 行に収まり、状態の表示（焦点・件数・絞り込み）が画面に 1 か所だけあり、凡例がグラフのノードを覆わない

## 本文

## 要求（2026-09-13、マスター承認済み。開発担当 deck へ振り出し）

根拠は D-41、D-44、D-45、D-39。状態は N-25 の後（f47b0c8）。判定は AC-31。受領はマスターと kuroko の目視を含む。

### 満たす条件

1. **3 群に分ける。** ヘッダー直下のツールバーは絞り込みだけを持つ: 検索、種別チップ、スコープ、要件状態、質問状態、criterion、右端にリセット。グラフ操作（近傍の ID と半径と実行、genealogy、ズーム −/+/Fit）はグラフ領域の上端の細い帯に移す。絞り込みの帯にグラフの状態を置かない。
2. **1 行に収まる。** 1440px で絞り込みの帯は 1 行、グラフ操作の帯も 1 行。1280px では折り返してよいが操作は隠れない。コントロールの高さと余白は `tokens.css` の値だけを使う（高さ 1 種類、余白は 4 の倍数の段階）。ラベルはコントロールの中（placeholder / 選択中の値）か直前の短い語で、同じ語を 2 回出さない（現状「Automatic radius」が select と文字の両方に出ている）。
3. **状態の表示は 1 か所。** 焦点（All nodes / 経路）、描画件数と非表示件数、絞り込みの要約を、グラフ操作の帯の 1 行にまとめる。「15 nodes drawn」バッジと下部トースト「N clusters · M nodes available」と「Types: ...」行は無くす。
4. **凡例はノードを覆わない。** グラフ領域の外（下端の帯か、折り畳める側面）に置く。
5. **ボタンの文言は結果が読める短さ。** 「Show 5 hops around D-37 (1 nodes before filters; up to 60 drawn)」のような文は、「D-37 の近傍 5 ホップを表示 · 1 件」のように対象・操作・件数の順で短くする。単複の文法も正す。
6. **状態は hash に載る**（D-38）。移した操作の状態も従来どおり復元される。

### 検証

- E2E: 1280 / 1440 / 1920 で、絞り込みの帯の全コントロールが 1 行（1440 / 1920）にあること、状態表示の要素が 1 つだけ存在すること、凡例の矩形とノードの矩形が交差しないこと。既存 28 件は通す（文言変更で期待値が変わるものは理由を D-44 / AC-31 として記す）。
- 目視: 1440px のスクリーンショットを `e2e/test-results/` に残し、報告に paths を書く。kuroko が terminal-browser で確認してから受領する。

### 共通の維持条件と完了

- 派生表示の契約、埋め込みエスケープ、単一 HTML・ネットワークなしは不変。互換な変更（z）。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`e2e` の `npm test`、`ui` の `bun run check` が exit 0。
- 受け入れ条件を evidence 付きで satisfy。need ごとにコミット、push と公開はマスターの指示待ち。報告は `herdr agent prompt kuroko` で D-33 の形、スクリーンショットの path を含める。判断に迷う点、原則（D-38〜D-45）と衝突する点は question として台帳に置いて私へ送る。

## 完了確認（kuroko、2026-09-14）

deck の報告をコミット 5697316 で照合。隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed、E2E 31 passed / 0 failed、bun run check exit 0。gy 自身の台帳（134 ノード）を新ビルドで描き、terminal-browser で目視: 絞り込みの帯 1 行（検索・種別チップ・スコープ・要件状態・質問・criterion・Reset all）、グラフ操作の帯 1 行（Node ID・Auto・Show neighbors・Lineage・−/+/Fit・Back/All/Read）、状態表示は「All nodes 15 clusters · 134 nodes 0 hidden No filters」の 1 行だけ、バッジとトーストと Types 行は消えた。凡例はグラフ下端の帯でノードを覆わない。焦点時の文言は「D-501: show 5 hops · 2 nodes」。受領。

目視で残った観察（N-26 の範囲外、後続へ）: 種別チップは全選択時に全て塗りつぶしで、押し込まれているのか単に有効なのか読めない（D-44 の「状態を自身に表示」、N-27）。グラフのラベルは依然として途中で切れる（N-24）。焦点ボタンがグラフ操作の帯と詳細の両方にある（同じ操作が 2 か所、N-27 で整理）。

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

