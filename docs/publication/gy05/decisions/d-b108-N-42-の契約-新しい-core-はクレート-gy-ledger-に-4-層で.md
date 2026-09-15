# d-b108 (D-78) N-42 の契約: 新しい core はクレート gy-ledger に 4 層で作り、永続化の実体は持たず、測る道具は scripts/measure.sh に置く

## 関係
- 無し

decision_scope: N-42 の依頼書（.local/brief-n42-draft.md）の「正本に無い 5 点」を lead が PO として決めた。2026-09-15。(1) 新しい core のクレートは crates/gy-ledger。今の gy-core は 0.4 の台帳を読む移行のためだけに残し、移行後に消す（D-76）。(2) N-42 は型と層の境界だけを作る。store は契約（トランザクション・ID・履歴・形式の版・GY_ACTOR）と in-memory の実体だけで、保存形式（未決 1）はこのニーズで決めない。(3) 測る道具は scripts/measure.sh。関数の長さは fn の行から対応する閉じ括弧までの行数（空行とコメント込み）。文字列キーは src/ で属性名の文字列リテラルを引数に get / insert / remove / contains_key / 添字で読み書きする行。差分は git diff --stat の追加+削除で Cargo.lock を除く。(4) 1 ニーズの差分 600 行にはテストと測る道具を含める。超えたら道具を別ニーズに割る。(5) piko のペインは無ければ herdr agent start piko --kind opencode で起こし、報告先は lead。適用範囲は N-42 と、その上に載る gy05 のニーズ。
scope: gy05
created: 2026-09-14
  spawns n-348d (N-45) 物理設計の基準を機械で測る道具 scripts/measure.sh を用意する（行数・関数の長さ・glob・文字列キー・テストの行数・差分・層の向き）
  spawns n-a603 (N-46) 新しい core の model 層: 5 種のノードの型と不変条件を src/model/ のディレクトリ分割で作る
  spawns n-bc0c (N-42) 新しい core の骨格を 4 層と型のモデルで作り、行数・glob・文字列キー・差分の大きさを測る道具を用意する
  spawns n-edce (N-47) 辺の種類の組（論点 closes 決定、ニーズ targets 受け入れ条件など）を model の不変条件として検査し、外れた組の Link を作れなくする

## Context

## Decision

## Consequences


