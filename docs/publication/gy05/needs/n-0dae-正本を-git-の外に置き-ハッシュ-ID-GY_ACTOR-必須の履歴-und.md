# n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ

state: closed
scope: gy05
created: 2026-09-14
  depends-on n-0cb7 (N-48) 正本の置き場所（XDG のデータディレクトリ + リポジトリのハッシュ）と形式の版 format の読み書き・判定・移行の入口の枠
  targets ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない
  targets ac-7670 (AC-45) 1 コマンドの書き込みは全部書けるか全部書かないか。途中失敗で中途半端な状態が残らない
  targets ac-b0f3 (AC-46) すべての書き込みに、いつ・誰が（GY_ACTOR 必須）・なぜ・出典が残り、undo できる
  targets ac-6251 (AC-50) 利用者の文書と運用に採番の規則が現れない。ID の衝突と振り直しが起きない
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている
  depended-on-by n-bd8d (N-49) undo（逆変更の行の追記）と snapshot.json（commit ごとの原子的な書き直し、無くてもログの再生で復元）
## 09-15 分割

見込み差分 850〜900 のため 3 つに分けた。このニーズに残るのは .local/brief-n38-draft.md の「作る」1〜4 と 8: log.rs（Event と changes、追記・読み・末尾の不完全な行の破棄）、FileStore（開く・transaction・commit・rollback・履歴・再オープン）、fs2 の排他ロック、seq の競合検出、ID の衝突で桁を増やす、model の serde derive、tests/file_store.rs と tests/concurrency.rs。changes の粒度はノード単位の全量（作成・属性・本文・辺・別名をまとめて 1 ノード分）で、行に node の ID と変更の種類（created / updated / deleted）を持ち、履歴を差分なしで読める形にする。置き場所と版は N-48、undo とスナップショットは N-49。

閉じた理由: 事実（migrated: complete）

