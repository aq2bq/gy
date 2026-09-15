gy は要求が確定するまでの状態をグラフで持ち、書き込みはすべて 1 トランザクションで追記する。不正は書いた時点で拒まれる。

セッション開始:
  gy handover
  gy next
  gy show <ID>

読み (5):
  gy show <ID|ref>... [--full]                         ノードを表示し、足りないものも出す
  gy list [--type] [--status] [--targets] [--grep] [--actor] [--since]   一覧。--actor / --since で書き込み単位
  gy next                                              前提の片付いたニーズ
  gy handover                                          進行中の要求と再開に要る件数
  gy publish [<ID>...] [--since] [--out]                既定は 1 ページ（判断待ち・未決・注意）。ID を渡すとそのノードの記述も出す

書き (15):
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
  gy decide "<題>" --scope-note <文> [--body-file <path>] [--closes <Q>]... [--relate <関係> <D> --mark <文>]
  gy link <from> <関係> <to> [--mark <文>] [--remove]
  gy edit <ID> --reason <文> [--title] [--body-file] [--set k=v] [--append k=v]
  gy undo --reason <文>

設定はスコープ名と出力先だけ:

  [scopes.myproject]
  output = "docs/gy.md"

書き込みの前に GY_ACTOR を名乗る。読みは名乗らなくてよい。
  export GY_ACTOR=<名前>
