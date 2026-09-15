# d-a3e2 (D-9) 近傍展開の辺集合は画面が描く EDGES ベースとする

- 種類: decision
- scope: html_projection
- created: 2026-09-13
- 別名: D-9

## 関係

- closed-by q-3d01 (Q-9) 近傍展開が使う辺集合は、画面が描く EDGES か、frontmatter の全参照か

## 成立範囲

展開先に画面上の辺を持たないノードが混ざると、展開したのに線が無いという画面と操作の不一致が出る。show と neighbors が EDGES を使う契約とも揃う。この結果 waiting-on と unresolved と belongs-to はグラフに現れず、詰まりの関係は Blockers パネルの一覧でのみ読める

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

