# n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割）

state: closed
scope: gy05
created: 2026-09-15
  depends-on n-8996 (N-59) 読み (2a): HistoryEntry の seq、need の状態の導出、next を views 層に作る
  spawned-by d-245c (D-75) 台帳は常に妥当とし、不正な状態は書き込み時に拒み、注意が要る状態は handover と next が出し、後から検査する lint を持たない
  targets ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  depended-on-by n-0471 (N-36) bearer_count と need の状態を導出にし、handover を error と進行中だけにする
  depended-on-by n-54fe (N-61) CLI (1a): 新しい gy の CLI クレート gy5 の骨格（clap、GY_ACTOR、--json、-C、終了コード、正本の自動作成）と読み 4 の配線
  depended-on-by n-da96 (N-39) 各コマンドの出力が、そのノードに無いものと次の操作を返し、同梱 skill を流れの説明だけにする


閉じた理由: 事実（migrated: complete）

