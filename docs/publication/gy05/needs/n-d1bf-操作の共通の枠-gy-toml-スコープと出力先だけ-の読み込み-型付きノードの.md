# n-d1bf (N-50) 操作の共通の枠: gy.toml（スコープと出力先だけ）の読み込み、型付きノードの読み書きと ID・別名の解決を担う Repository、1 意図 1 トランザクションの実行の型、出力の型（ID・変えた項目・無いもの・次の操作）

state: closed
scope: gy05
created: 2026-09-15
  depends-on n-bd8d (N-49) undo（逆変更の行の追記）と snapshot.json（commit ごとの原子的な書き直し、無くてもログの再生で復元）
  spawned-by d-f77c (D-69) 新しい gy の操作は末端 20（書き 14、読み 6）の閉じた集合とし、一つの意図を一つのコマンド・一つのトランザクションで行う
  targets ac-9be9 (AC-40) gy.toml に書けるのはスコープと出力先だけで、他のキーは読み込み時にエラー。Kokopelli の gy.toml 2,117 行が 16 行相当で読み込める
  targets ac-7670 (AC-45) 1 コマンドの書き込みは全部書けるか全部書かないか。途中失敗で中途半端な状態が残らない
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  depended-on-by n-447c (N-54) 操作 (d): edit（--title / --body / --set / --append、--reason 必須。状態と辺は変えない）と undo の操作
  depended-on-by n-81d8 (N-52) 操作 (b): decide（ADR 有り無し、closes / narrows / supersedes / completes / widens と mark）と link（--mark、--remove）
  depended-on-by n-d1c6 (N-53) 操作 (c1): req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）と、resolve の ref 対応
  depended-on-by n-d514 (N-51) 操作 (a1): 辺を from 側だけに置き Need.targets の重複を消し、need add / need close を作る
  depended-on-by n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い）
## 経緯

store（N-42、N-38、N-48、N-49）と model（N-46、N-47）が揃ったので、操作（N-37）の前に ops 層の共通の枠を 1 ニーズとして立てた（2026-09-15 lead）。N-35 の「gy.toml をスコープと出力先だけにする」の新しい core 側はこのニーズで満たす（AC-40）。

## 範囲

1. gy.toml の読み込み: `[scopes.<名前>]` と `output`（publish の出力先）だけ。他のキーは読み込み時に Err（AC-40）。
2. Node に scope と created を足す（model の小さな変更）。
3. Repository: store（FileStore / MemoryStore）の上で、型付き Node の get / put / remove、ID の解決（完全な ID、0 埋めの同一視、別名、後で ref）、トランザクションの実行。
4. 操作の型: Intent（入力）→ run(repo) → Outcome { id, changed, missing, next }。
5. N-37 の分割はこの枠の上で行う。

閉じた理由: 事実（migrated: complete）

