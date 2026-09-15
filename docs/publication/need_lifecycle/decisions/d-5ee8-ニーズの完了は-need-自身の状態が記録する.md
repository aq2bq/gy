# d-5ee8 (D-28) ニーズの完了は need 自身の状態が記録する

- 種類: decision
- scope: need_lifecycle
- created: 2026-09-13
- 別名: D-28

## 関係

- closed-by q-4b50 (Q-19) 要求にしないまま終わったニーズを next から外す手段が無い
- spawns n-44d8 (N-14) ニーズのライフサイクルを設計として定義し、文書と同梱スキルに載せる
- spawns n-fe2d (N-15) 定義された状態のいずれでもないニーズの状態を lint が検出する

## 成立範囲

gy の台帳一般に適用する。need の status が complete であるか、filed-as の先の要求がすべて完了しているとき、その need は完了として next から外す。この判定は 0.3.1 の実装に既に存在し（views.rs の need_done）、GitHub Issue を使わない運用でも node set で記録できることを再現で確認した。したがって要求を経ない完了の記録手段を新たに追加せず、利用者側にも追加の作業を残さない。欠けているのは need の状態を設計として定義すること、定義外の値を lint が検出すること、文書と同梱スキルに載せることである。need の完了に証跡を要求する専用の操作は足さない

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

