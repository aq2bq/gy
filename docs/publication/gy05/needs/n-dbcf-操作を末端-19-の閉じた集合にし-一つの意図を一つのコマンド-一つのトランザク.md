# n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い）

state: closed
scope: gy05
created: 2026-09-14
  depends-on n-d1bf (N-50) 操作の共通の枠: gy.toml（スコープと出力先だけ）の読み込み、型付きノードの読み書きと ID・別名の解決を担う Repository、1 意図 1 トランザクションの実行の型、出力の型（ID・変えた項目・無いもの・次の操作）
  depends-on n-d514 (N-51) 操作 (a1): 辺を from 側だけに置き Need.targets の重複を消し、need add / need close を作る
  depends-on n-81d8 (N-52) 操作 (b): decide（ADR 有り無し、closes / narrows / supersedes / completes / widens と mark）と link（--mark、--remove）
  depends-on n-d1c6 (N-53) 操作 (c1): req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）と、resolve の ref 対応
  depends-on n-447c (N-54) 操作 (d): edit（--title / --body / --set / --append、--reason 必須。状態と辺は変えない）と undo の操作
  depends-on n-816a (N-55) 操作 (a2): question add / question close / criterion add / criterion satisfy の 4 操作（N-51 から分割）
  depends-on n-c980 (N-56) 操作 (c2): req approve（--design、--heard-by、--evidence）/ revise / done / cancel と、確定後の 3 つの記録（N-53 から分割）
  depends-on n-f312 (N-62) CLI (2a): decide / link / edit / undo の配線と、書きの入口での actor 検査の前倒し
  depends-on n-478b (N-64) CLI (2b): req add / approve / revise / done / cancel の配線と末端の数の測定（AC-49）（N-62 から分割）
  targets ac-09b1 (AC-49) 末端のサブコマンドが 20 以下（書き 15、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける
  targets ac-7670 (AC-45) 1 コマンドの書き込みは全部書けるか全部書かないか。途中失敗で中途半端な状態が残らない
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている
## 09-15 分割の計画（lead）

共通の枠は N-50。その上で 4 つに割る予定（各 600 行以下）: (a) need add / need close / question add / question close / criterion add / criterion satisfy、(b) decide（ADR 有り無し、closes / narrows / supersedes / completes / widens）と link（--mark、--remove）、(c) req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）/ approve / revise / done / cancel、(d) edit（--reason 必須。状態と辺は変えない）と undo。読み 5（show / list / next / handover / publish）は N-39 / N-36 / N-40 の側で扱う。CLI と MCP の配線は別ニーズ。

閉じた理由: 事実（migrated: complete）

