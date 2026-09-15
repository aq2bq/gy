# n-3d8c (N-35) workflow.records / guards / スナップショット / lint 設定 / import 設定 / 申告オプション / gate を消し、gy.toml をスコープと出力先だけにする

- 種類: need
- scope: gy05
- created: 2026-09-14
- 状態: closed
- 別名: N-35

## 関係

- targets ac-5216 (AC-41) 確定後の転記が 0。進行管理が外の文書から gy へ写す記録が無い（今日は #6027 で約 22,800 bytes）
- targets ac-9be9 (AC-40) gy.toml に書けるのはスコープと出力先だけで、他のキーは読み込み時にエラー。Kokopelli の gy.toml 2,117 行が 16 行相当で読み込める
- targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
- targets ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%）

## 本文

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

