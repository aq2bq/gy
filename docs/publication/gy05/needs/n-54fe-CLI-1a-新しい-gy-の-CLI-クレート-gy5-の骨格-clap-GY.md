# n-54fe (N-61) CLI (1a): 新しい gy の CLI クレート gy5 の骨格（clap、GY_ACTOR、--json、-C、終了コード、正本の自動作成）と読み 4 の配線

state: closed
scope: gy05
created: 2026-09-15
  depends-on n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割）
  spawned-by d-f77c (D-69) 新しい gy の操作は末端 20（書き 14、読み 6）の閉じた集合とし、一つの意図を一つのコマンド・一つのトランザクションで行う
  targets ac-09b1 (AC-49) 末端のサブコマンドが 20 以下（書き 15、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける
  targets ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  depended-on-by n-f312 (N-62) CLI (2a): decide / link / edit / undo の配線と、書きの入口での actor 検査の前倒し
  depended-on-by n-f65b (N-63) CLI (1b): need / question / criterion の 6 操作の配線とテスト（N-61 から分割）
## 経緯

読み（N-57〜N-60）が揃うので CLI を配線する（2026-09-15 lead）。0.4 の gy バイナリは移行（N-41）が終わるまで自分の台帳の操作に要るので、新しい CLI は別クレート crates/gy5（バイナリ名 gy5）として作り、0.5.0 の準備で crates/gy を置き換える。init は作らず、正本のディレクトリが無ければ最初の書き込みで作る。

## 09-15 分割

見込み約 920 行のため 6 操作の配線を N-63 に割った（ピコちゃんの案）。このニーズは骨格と読み 4 と tests/common, reads。

閉じた理由: 事実（migrated: complete）

