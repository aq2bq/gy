# n-c980 (N-56) 操作 (c2): req approve（--design、--heard-by、--evidence）/ revise / done / cancel と、確定後の 3 つの記録（N-53 から分割）

state: closed
scope: gy05
created: 2026-09-15
  depends-on n-d1c6 (N-53) 操作 (c1): req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）と、resolve の ref 対応
  spawned-by d-be4c (D-70) 確定後の要求について gy が持つのは、承認後の改訂・完了・中止の 3 つの人による記録だけであり、gy は GitHub を読まない
  targets ac-09b1 (AC-49) 末端のサブコマンドが 20 以下（書き 15、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  targets ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%）
  depended-on-by n-447c (N-54) 操作 (d): edit（--title / --body / --set / --append、--reason 必須。状態と辺は変えない）と undo の操作
  depended-on-by n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い）
## 経緯

N-53 で 5 操作を入れると score 911（基準 600）のため、事前に決めた分け方で approve / revise / done / cancel を分けた（2026-09-15）。退避は .local/wip-req/。範囲は .local/brief-n53.md の表の 4 行と、model の Requirement に approval / revisions / completion / cancellation を型で持たせること、テスト 4 ファイル。N-54（edit と undo）はこの後。

閉じた理由: 事実（migrated: complete）

