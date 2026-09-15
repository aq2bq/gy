# ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す

satisfied: true (2026-09-15 N-46 / N-47 / N-52 / N-60。不正は構築時と操作の入口で Err（空題名、成立範囲なし、許されない遷移、種類の組に無い辺、mark が本文に無い、二重の link、同じ ref の未完了、予約語の edit）。壊れたログの行は開くときに Err。lint は無く、注意が要る状態は handover が件数で出す（D-75）) at 2026-09-15T03:51:52.084975+00:00
scope: gy05
created: 2026-09-14
  targeted-by n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割）
  targeted-by n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ
  targeted-by n-1aad edit --set scope=<名前> でノードを別のスコープへ移せるようにする（名前は gy.toml にあるものだけ。ID と辺は変わらない）
  targeted-by n-35cf show / list / publish のニーズの状態を next と同じ導出（open / closed / done）にする。list --status done を受ける
  targeted-by n-447c (N-54) 操作 (d): edit（--title / --body / --set / --append、--reason 必須。状態と辺は変えない）と undo の操作
  targeted-by n-816a (N-55) 操作 (a2): question add / question close / criterion add / criterion satisfy の 4 操作（N-51 から分割）
  targeted-by n-81d8 (N-52) 操作 (b): decide（ADR 有り無し、closes / narrows / supersedes / completes / widens と mark）と link（--mark、--remove）
  targeted-by n-8996 (N-59) 読み (2a): HistoryEntry の seq、need の状態の導出、next を views 層に作る
  targeted-by n-a603 (N-46) 新しい core の model 層: 5 種のノードの型と不変条件を src/model/ のディレクトリ分割で作る
  targeted-by n-bc0c (N-42) 新しい core の骨格を 4 層と型のモデルで作り、行数・glob・文字列キー・差分の大きさを測る道具を用意する
  targeted-by n-c980 (N-56) 操作 (c2): req approve（--design、--heard-by、--evidence）/ revise / done / cancel と、確定後の 3 つの記録（N-53 から分割）
  targeted-by n-d1bf (N-50) 操作の共通の枠: gy.toml（スコープと出力先だけ）の読み込み、型付きノードの読み書きと ID・別名の解決を担う Repository、1 意図 1 トランザクションの実行の型、出力の型（ID・変えた項目・無いもの・次の操作）
  targeted-by n-d1c6 (N-53) 操作 (c1): req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）と、resolve の ref 対応
  targeted-by n-d514 (N-51) 操作 (a1): 辺を from 側だけに置き Need.targets の重複を消し、need add / need close を作る
  targeted-by n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い）
  targeted-by n-edce (N-47) 辺の種類の組（論点 closes 決定、ニーズ targets 受け入れ条件など）を model の不変条件として検査し、外れた組の Link を作れなくする
  targeted-by n-f088 ID の解決で、0 埋めの同一視を旧 ID の別名にだけ当て、gy が振ったハッシュ ID には当てない（d-0008 が D-8 と衝突する欠陥）。新しい ID は数字だけのハッシュを避ける



