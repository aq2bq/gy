gy は要求が確定するまでの状態をグラフで持ち、書き込みはすべて 1 トランザクションで追記する。不正は書いた時点で拒まれる。

セッション開始:
  gy handover
  gy next
  gy show <ID>

読み (6):
  gy show <ID|ref>... [--full]                         ノードを表示し、足りないものも出す
  gy list [--type] [--status] [--targets] [--grep] [--actor] [--since]   一覧。--actor / --since で書き込み単位
  gy next                                              前提の片付いたニーズ
  gy handover                                          進行中の要求と再開に要る件数
  gy publish [--scope] [--since] [--out]                記録の公開物（範囲内の全ノードの逐語・履歴・診断）。コミットして後から振り返る
  gy serve                                              台帳をブラウザで読む。127.0.0.1、GET だけ、書く経路は無い。止めるまで。端末から起動したときはブラウザを開く

書き (16):
  gy need add "<題>" --targets <AC>... [--spawned-by <D>]
  gy need close <ID> --by fact|external --evidence <文>
  gy question add "<題>" --decider <名> --options <文>...        選択肢は 2 つ以上
  gy question close <ID> --by fact|decision|non-decision --evidence <文> [--decision <D>]
  gy criterion add "<題>"
  gy criterion satisfy <AC> --evidence <文> [--revoke]
  gy req add "<題>" --need <N>... [--relies-on <D>]... [--targets <AC>]... [--ref <URL>]
  gy req approve <ID|ref> --design <文> --heard-by <名> --evidence <文>
  gy req revise <ID> --reason <文> --source <文>
  gy req done <ID> --evidence <文>
  gy req cancel <ID> --reason <文> --source <文>
  gy decide "<題>" --scope-note <文> [--body-file <path>] [--closes <Q>]... [--relate <関係> <D> --mark <文>] [--source <文>]
  gy link <from> <関係> <to> [--mark <文>] [--remove]
  gy edit <ID> --reason <文> [--title] [--body-file] [--set k=v] [--append k=v]
        自由属性は文字列。--set は上書き、--set k= は消去、--append は改行区切りで 1 行足す
        --set scope=<名前> で gy.toml にあるスコープへ移動。--set decision_scope=<文> で未記録の成立範囲を 1 回だけ記録
  gy scope rename <旧> <新>
        旧スコープの全ノードを新名へ移し、gy.toml をコメントと順序を保ったまま書き換える
  gy undo --reason <文>
        直前の 1 件だけを戻す。続けて打つと undo 自身を戻す（redo）。2 件以上戻す操作は無い

設定はスコープ名と出力先だけ:

  [scopes.myproject]
  output = "docs/gy.md"

書き込みの前に GY_ACTOR を名乗る。読みは名乗らなくてよい。
  export GY_ACTOR=<名前>
