# d-245c (D-75) 台帳は常に妥当とし、不正な状態は書き込み時に拒み、注意が要る状態は handover と next が出し、後から検査する lint を持たない

- 種類: decision
- scope: roadmap
- created: 2026-09-14
- 別名: D-75

## 関係

- narrows d-f77c (D-69) 新しい gy の操作は末端 20（書き 14、読み 6）の閉じた集合とし、一つの意図を一つのコマンド・一つのトランザクションで行う（mark: 末端 20（書き 14、読み 6））
- spawns n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割）
- spawns n-8996 (N-59) 読み (2a): HistoryEntry の seq、need の状態の導出、next を views 層に作る

## 成立範囲

新しい gy の整合の担保。2026-09-15 マスターの問い。書ける経路が gy だけであることが前提。store 自体の整合の確認と回復は handover の起動時に自動で行う。操作は末端 19。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

