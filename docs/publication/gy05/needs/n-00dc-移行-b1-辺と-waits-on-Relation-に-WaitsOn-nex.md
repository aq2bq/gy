# n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）

- 種類: need
- scope: gy05
- created: 2026-09-15
- 状態: closed
- 別名: N-66

## 関係

- depended-on-by n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）
- depended-on-by n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正
- depends-on n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）
- depends-on n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割）
- spawned-by d-9751 (D-82) 新しい gy の正本は追記専用の JSONL イベントログ + スナップショット（ファイル、依存ゼロ）で持つ
- targets ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ
- targets ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0
- targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
- targets ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%）

## 本文

## 09-15 分割

見込み約 905 行のため、要求の状態・凍結・報告・gy.toml を N-68 に割った（ピコちゃんの案）。このニーズは辺の写しと waits-on（gy-ledger の WaitsOn、derive.rs の辺読み、gy5/src/write.rs の canonical 名に waits-on を 1 行）と tests/edges.rs。許可した追加: crates/gy5/src/write.rs の 1 行。

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

