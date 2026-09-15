# d-be4c (D-70) 確定後の要求について gy が持つのは、承認後の改訂・完了・中止の 3 つの人による記録だけであり、gy は GitHub を読まない

- 種類: decision
- scope: roadmap
- created: 2026-09-14
- 別名: D-70

## 関係

- narrows d-9efb (D-59) gy が扱うのは要求が確定するまでであり、確定した要求からは GitHub の Issue でよい（mark: 確定した要求からは GitHub の Issue でよい）
- spawns n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割）
- spawns n-c980 (N-56) 操作 (c2): req approve（--design、--heard-by、--evidence）/ revise / done / cancel と、確定後の 3 つの記録（N-53 から分割）
- spawns n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正

## 成立範囲

要求の確定後の扱い。二人の批評の最大の反対理由（確定・未完了が見えない）への回答。D-59（確定後は Issue でよい）を狭める。状態は起票済み・確定・完了・中止の 4 つ。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

