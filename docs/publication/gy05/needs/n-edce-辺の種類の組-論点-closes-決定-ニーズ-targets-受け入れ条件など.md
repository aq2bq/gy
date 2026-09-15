# n-edce (N-47) 辺の種類の組（論点 closes 決定、ニーズ targets 受け入れ条件など）を model の不変条件として検査し、外れた組の Link を作れなくする

state: closed
scope: gy05
created: 2026-09-15
  depends-on n-a603 (N-46) 新しい core の model 層: 5 種のノードの型と不変条件を src/model/ のディレクトリ分割で作る
  spawned-by d-b108 (D-78) N-42 の契約: 新しい core はクレート gy-ledger に 4 層で作り、永続化の実体は持たず、測る道具は scripts/measure.sh に置く
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
## 経緯

N-46 の検収で、Relation を閉じた enum にする差し戻しをした際、種類の組の検査は score 600 の都合で次に回した（2026-09-15）。D-75「不正は書き込み時に拒む」の一部であり、model に置く（D-76: 同じ概念を 2 か所に書かない）。

## 範囲

0.4 の Directions から gate を除いた組: question closes decision / decision narrows|widens|supersedes|completes decision / need|requirement targets criterion / need spawned-by decision / need filed-as requirement / need depends-on need / requirement relies-on decision / requirement raised question。Link::new を Result にし、from と to の種類が組に無ければ Err。Relation ごとの許される (from, to) の表は model に 1 か所。

閉じた理由: 事実（migrated: complete）

