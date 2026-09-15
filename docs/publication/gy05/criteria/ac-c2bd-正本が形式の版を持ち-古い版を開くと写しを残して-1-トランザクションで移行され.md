# ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている

- 種類: criterion
- scope: gy05
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-55

## 関係

- targeted-by n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）
- targeted-by n-0cb7 (N-48) 正本の置き場所（XDG のデータディレクトリ + リポジトリのハッシュ）と形式の版 format の読み書き・判定・移行の入口の枠
- targeted-by n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ
- targeted-by n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割）
- targeted-by n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）
- targeted-by n-66ee (N-34) 要求の状態を 4 つにし、確定後は改訂・完了・中止の記録だけを持ち、ID を振り直して ref を持たせる
- targeted-by n-bc0c (N-42) 新しい core の骨格を 4 層と型のモデルで作り、行数・glob・文字列キー・差分の大きさを測る道具を用意する
- targeted-by n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い）

## 本文

## 充足

- satisfied（2026-09-15 N-48 / N-41。正本は生まれた時に format=1 を持ち、無い・新しすぎる・壊れた行は開けない（tests/format.rs、file_store.rs）。非互換の変更（waits-on の追加 D-83、閉じた関係、4 状態）は移行 N-65〜N-69 に同梱。Kokopelli の写しで試した値は AC-47 の evidence と .local/report-n41.md） 2026-09-15T03:51:51.019324+00:00

## 自由属性

- 無し

