# n-0cb7 (N-48) 正本の置き場所（XDG のデータディレクトリ + リポジトリのハッシュ）と形式の版 format の読み書き・判定・移行の入口の枠

state: closed
scope: gy05
created: 2026-09-15
  spawned-by d-9751 (D-82) 新しい gy の正本は追記専用の JSONL イベントログ + スナップショット（ファイル、依存ゼロ）で持つ
  targets ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない
  targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  depended-on-by n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ
## 経緯

N-38 の見込み差分が約 850〜900 行（基準 600）のため、2026-09-15 にピコちゃんの 3 分割案を採った。順序は N-48（置き場所と版）→ N-38（ログと回復、ロックと競合、ID）→ N-49（undo とスナップショット）。

## 範囲

.local/brief-n38-draft.md の「作る」7 と 9、契約の AC-44 と AC-55、tests/format.rs。location.rs（XDG パスとルートのハッシュ）と format の読み書き・supported・migrate(from, to) の枠。

閉じた理由: 事実（migrated: complete）

