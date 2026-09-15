# ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%）

- 種類: criterion
- scope: gy05
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-42

## 関係

- targeted-by n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）
- targeted-by n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割）
- targeted-by n-3d8c (N-35) workflow.records / guards / スナップショット / lint 設定 / import 設定 / 申告オプション / gate を消し、gy.toml をスコープと出力先だけにする
- targeted-by n-66ee (N-34) 要求の状態を 4 つにし、確定後は改訂・完了・中止の記録だけを持ち、ID を振り直して ref を持たせる
- targeted-by n-c980 (N-56) 操作 (c2): req approve（--design、--heard-by、--evidence）/ revise / done / cancel と、確定後の 3 つの記録（N-53 から分割）
- targeted-by n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正

## 本文

## 測り方

移行後の要求ノードを show --full し、確定前の記述・辺・ref・approval / revisions / completion / cancellation 以外の記録が無いことを見る。確定後の記録は publication に凍結され legacy_records の相対パスだけが残る。

## 充足

- satisfied（2026-09-15 N-68 / N-69 / N-41。移行後の要求ノードは確定前の記述（題名・本文・辺・ref・自由属性）と approval / revisions / completion / cancellation の記録だけ。確定後の記録 37 件は publication に凍結し legacy_records の相対パスだけ残す。0.4 の 431 KB の要求ファイルに対し events.jsonl の要求は 1 ノード 1 JSON） 2026-09-15T03:51:51.153329+00:00

## 自由属性

- 無し

