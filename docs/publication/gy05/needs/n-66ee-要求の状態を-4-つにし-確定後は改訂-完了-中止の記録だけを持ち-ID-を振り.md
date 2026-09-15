# n-66ee (N-34) 要求の状態を 4 つにし、確定後は改訂・完了・中止の記録だけを持ち、ID を振り直して ref を持たせる

state: closed
scope: gy05
created: 2026-09-14
  targets ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%）
  targets ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ
  targets ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている
## 09-15 進捗の整理（lead）

model の側は N-46 で済んだ: 要求の状態は Filed / Approved / Done / Cancelled の 4 値、許される遷移は 5 つ、ref は不透明な Option<Ref>。残るのは (1) 操作 req add --ref / approve / revise / done / cancel（N-37 の分割で扱う）、(2) handover に確定・未完了の要求を ref 付きで出す（views、N-36 / N-39 の分割で扱う）、(3) Issue 番号の ID を gy の ID に振り直し ref に URL を持たせる移行（N-41）。このニーズ自体は (1)〜(3) が揃った時点で AC-42 / AC-43 / AC-48 を測って閉じる。

閉じた理由: 事実（migrated: complete）

