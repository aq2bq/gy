# n-bc0c (N-42) 新しい core の骨格を 4 層と型のモデルで作り、行数・glob・文字列キー・差分の大きさを測る道具を用意する

state: closed
scope: gy05
created: 2026-09-14
  spawned-by d-b108 (D-78) N-42 の契約: 新しい core はクレート gy-ledger に 4 層で作り、永続化の実体は持たず、測る道具は scripts/measure.sh に置く
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている
  depended-on-by n-a603 (N-46) 新しい core の model 層: 5 種のノードの型と不変条件を src/model/ のディレクトリ分割で作る
## 09-15 分割

見込み差分が 600 を超えたため、測る道具を N-45 に割った（D-78）。

## 09-15 二度目の分割

実測 score 817 のため、model 層（5 種の型と不変条件）を N-46 に割った。N-42 に残るのは lib（4 層の宣言と名指し pub use）、store（契約と in-memory）、ops / views の骨、最小の model（NodeKind と Node の identity）、store のテスト。目標は 600 以下。

閉じた理由: 事実（migrated: complete）

