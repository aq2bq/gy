# n-f65b (N-63) CLI (1b): need / question / criterion の 6 操作の配線とテスト（N-61 から分割）

- 種類: need
- scope: gy05
- created: 2026-09-15
- 状態: closed
- 別名: N-63

## 関係

- depended-on-by n-f312 (N-62) CLI (2a): decide / link / edit / undo の配線と、書きの入口での actor 検査の前倒し
- depends-on n-54fe (N-61) CLI (1a): 新しい gy の CLI クレート gy5 の骨格（clap、GY_ACTOR、--json、-C、終了コード、正本の自動作成）と読み 4 の配線
- spawned-by d-f77c (D-69) 新しい gy の操作は末端 20（書き 14、読み 6）の閉じた集合とし、一つの意図を一つのコマンド・一つのトランザクションで行う
- targets ac-09b1 (AC-49) 末端のサブコマンドが 20 以下（書き 15、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける
- targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている

## 本文

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

