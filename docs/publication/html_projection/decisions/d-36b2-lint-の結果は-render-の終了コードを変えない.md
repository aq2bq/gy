# d-36b2 (D-6) lint の結果は render の終了コードを変えない

- 種類: decision
- scope: html_projection
- created: 2026-09-13
- 別名: D-6

## 関係

- closed-by q-7234 (Q-6) render の終了コードは lint の結果を反映すべきか

## 成立範囲

診断は投影されるデータであって生成の成否ではない。重大度で終了コードが変われば [lint] の設定変更だけで生成が失敗扱いになる。lint が error を報告する台帳でもHTMLを生成する。整合が崩れている台帳ほど俯瞰が要るため

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

