# n-81d8 (N-52) 操作 (b): decide（ADR 有り無し、closes / narrows / supersedes / completes / widens と mark）と link（--mark、--remove）

- 種類: need
- scope: gy05
- created: 2026-09-15
- 状態: closed
- 別名: N-52

## 関係

- depended-on-by n-d1c6 (N-53) 操作 (c1): req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）と、resolve の ref 対応
- depended-on-by n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い）
- depends-on n-816a (N-55) 操作 (a2): question add / question close / criterion add / criterion satisfy の 4 操作（N-51 から分割）
- depends-on n-d1bf (N-50) 操作の共通の枠: gy.toml（スコープと出力先だけ）の読み込み、型付きノードの読み書きと ID・別名の解決を担う Repository、1 意図 1 トランザクションの実行の型、出力の型（ID・変えた項目・無いもの・次の操作）
- depends-on n-d514 (N-51) 操作 (a1): 辺を from 側だけに置き Need.targets の重複を消し、need add / need close を作る
- spawned-by d-f77c (D-69) 新しい gy の操作は末端 20（書き 14、読み 6）の閉じた集合とし、一つの意図を一つのコマンド・一つのトランザクションで行う
- targets ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける
- targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
- targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている

## 本文

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

