# d-36b2 (D-6) lint の結果は render の終了コードを変えない

## 関係
- 無し

decision_scope: 診断は投影されるデータであって生成の成否ではない。重大度で終了コードが変われば [lint] の設定変更だけで生成が失敗扱いになる。lint が error を報告する台帳でもHTMLを生成する。整合が崩れている台帳ほど俯瞰が要るため
scope: html_projection
created: 2026-09-13
  closed-by q-7234 (Q-6) render の終了コードは lint の結果を反映すべきか

## Context

## Decision

## Consequences


