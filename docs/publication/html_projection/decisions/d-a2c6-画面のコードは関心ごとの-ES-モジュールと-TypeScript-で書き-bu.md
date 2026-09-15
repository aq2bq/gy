# d-a2c6 (D-45) 画面のコードは関心ごとの ES モジュールと TypeScript で書き、bun で 1 本に束ねた成果物を html.rs が埋め込む

- 種類: decision
- scope: html_projection
- created: 2026-09-13
- 別名: D-45

## 関係

- spawns n-3f2d (N-26) ツールバーと状態表示を設計トークンに基づいて組み直し、絞り込み・グラフ操作・リセットを分ける
- spawns n-b4d3 (N-25) 画面のコードを ui/ 配下の TypeScript モジュールに再構成し、bun で束ねた成果物を html.rs が埋め込む形にする

## 成立範囲

crates/gy-core の HTML projection。ソースは ui/ 配下に state（状態と hash codec の唯一の定義）、絞り込み、一覧、詳細、グラフ、状況把握、共通の設計トークン（余白・文字寸法・色）とコンポーネントに分ける。bun build の成果物（1 JS + 1 CSS）はリポジトリにコミットし、cargo build は bun を要求しない。CI は bun build を再実行して成果物が一致することを検査する。出力は従来どおり単一の HTML でネットワークへ出ない。E2E（e2e/）が振る舞いの契約であり、再構成の前後で同じテストが通る。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

