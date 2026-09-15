# ac-a8ff (AC-51) Kokopelli の TEAM_AGENTS.md の gy に関する行数が、gy の出力が次の操作と不足を返すことで減る

- 種類: criterion
- scope: gy05
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-51

## 関係

- targeted-by n-da96 (N-39) 各コマンドの出力が、そのノードに無いものと次の操作を返し、同梱 skill を流れの説明だけにする

## 本文

## 測り方

Kokopelli の TEAM_AGENTS.md の gy に関する行（grep -ci gy）を移行前と書き直し後で比べる。

## 充足

- satisfied（2026-09-15 lead が測定。Kokopelli の TEAM_AGENTS.md の gy に関する行（grep -ci gy）は移行前（bdedc1080）22 行、gy 0.5 の運用に書き直した後（1006ec8fd）20 行。全体は 175 → 172 行。減り幅は小さいが、records / guards / lint / cheatsheet の手順は消え、事実を得た担当が GY_ACTOR 付きで書き進行管理が list --since で照合する運用になった。Kokopelli の進行管理担当の完了報告（後始末: 決定 177 件の本文整理、未記録の成立範囲 81 件と空の AC 11 件の記入、scope rename、gy.toml をルートへ、docs/adr の削除、docs/gy-published への publish）） 2026-09-15

## 自由属性

- 無し

