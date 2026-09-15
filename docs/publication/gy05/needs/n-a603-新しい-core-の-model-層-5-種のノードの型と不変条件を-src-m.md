# n-a603 (N-46) 新しい core の model 層: 5 種のノードの型と不変条件を src/model/ のディレクトリ分割で作る

state: closed
scope: gy05
created: 2026-09-15
  depends-on n-bc0c (N-42) 新しい core の骨格を 4 層と型のモデルで作り、行数・glob・文字列キー・差分の大きさを測る道具を用意する
  spawned-by d-b108 (D-78) N-42 の契約: 新しい core はクレート gy-ledger に 4 層で作り、永続化の実体は持たず、測る道具は scripts/measure.sh に置く
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  depended-on-by n-edce (N-47) 辺の種類の組（論点 closes 決定、ニーズ targets 受け入れ条件など）を model の不変条件として検査し、外れた組の Link を作れなくする
## 経緯

N-42 の実装を measure.sh で測ると score 817（基準 600）で、超過の大半が model（src/model.rs 364 行 + tests/model.rs 120 行）だった。依頼書の「なお超えるなら model を先に別ニーズへ割る」に従い、2026-09-15 にピコちゃんの案 (A) を採って分けた。

## 範囲

.local/brief-n42-draft.md の「実装契約 2. model」をそのまま移す。model.rs は 300 行を超えるので src/model/ のディレクトリ分割にする（measure.sh はディレクトリ形を層として扱う）。N-42 には NodeKind と Node の identity だけの最小 model（約 40 行）が残り、このニーズがそれを本実装に置き換える。

閉じた理由: 事実（migrated: complete）

