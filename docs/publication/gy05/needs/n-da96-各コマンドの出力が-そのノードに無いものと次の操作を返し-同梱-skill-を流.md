# n-da96 (N-39) 各コマンドの出力が、そのノードに無いものと次の操作を返し、同梱 skill を流れの説明だけにする

- 種類: need
- scope: gy05
- created: 2026-09-14
- 状態: closed
- 別名: N-39

## 関係

- depends-on n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割）
- depends-on n-58b4 (N-58) 読み (1b): list（--type / --status / --targets / --grep / --actor / --since。actor と since は書き込み単位）を views 層に作る（N-57 から分割）
- depends-on n-7cc1 (N-57) 読み (1a): views の骨格と show（複数 ID、ref でも引ける、種類ごとの逐語、--full、--json）
- depends-on n-8996 (N-59) 読み (2a): HistoryEntry の seq、need の状態の導出、next を views 層に作る
- targets ac-a8ff (AC-51) Kokopelli の TEAM_AGENTS.md の gy に関する行数が、gy の出力が次の操作と不足を返すことで減る
- targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている

## 本文

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

