# n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）

- 種類: need
- scope: gy05
- created: 2026-09-15
- 状態: closed
- 別名: N-65

## 関係

- depended-on-by n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）
- depended-on-by n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割）
- depended-on-by n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）
- depends-on n-478b (N-64) CLI (2b): req add / approve / revise / done / cancel の配線と末端の数の測定（AC-49）（N-62 から分割）
- spawned-by d-8637 (D-83) ニーズが待っている論点は閉じた関係 waits-on（need → question）で持ち、書き込み時に検証する。自由属性 waiting-on は移行で辺に写す
- spawned-by d-9751 (D-82) 新しい gy の正本は追記専用の JSONL イベントログ + スナップショット（ファイル、依存ゼロ）で持つ
- targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている
- targets ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0
- targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている

## 本文

## 09-15 分割

見込み約 630 行のためノードの写しを N-67 に割った（ピコちゃんの案）。方針: gy-core の文字列キーを読むのは legacy.rs の 1 ファイルに閉じ、nodes.rs は型付きフィールドだけを読む。measure の文字列キーはこのクレートでは対象外（依頼書に明記）。

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

