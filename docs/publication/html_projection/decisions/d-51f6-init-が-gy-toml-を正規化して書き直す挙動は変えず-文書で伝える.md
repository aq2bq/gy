# d-51f6 (D-11) init が gy.toml を正規化して書き直す挙動は変えず、文書で伝える

## 関係
- 無し

decision_scope: 設定の値は保たれ機能上の損失がない。書式の保存には TOML の round-trip 編集が必要で 0.3.0 の範囲を超える。コメントと書式が失われることと、コメントを残しているなら .gitignore へ手で1行足すほうが安全であることを CHANGELOG と両READMEに書く
scope: html_projection
created: 2026-09-13
  closed-by q-4dd9 (Q-11) init が既存の gy.toml を正規化して書き直す挙動を変えるか

## Context

## Decision

## Consequences


