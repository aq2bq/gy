# n-7681 (N-32) ワークフローの外で進んだ要求 (legacy) を、証跡付きで complete へ進められる経路を用意する

state: open
scope: guard_waiver
created: 2026-09-14
  spawned-by d-237a (D-57) ガードの免除は gy.toml の waived_by が指す記録で表し、有効な免除記録を持つ要求にはそのガードの records と checks を掛けず、snapshot に免除の事実を残す
  targets ac-7145 (AC-37) ワークフローの外で閉じた要求を、実在しない作業記録を作らずに証跡付きで complete へ進められ、ワークフローを通る要求の complete のガードは変わらず、完了後の lint / handover が履歴を再検査しても指摘を出さない
## 経緯

Kokopelli の進行担当からマスター経由で受けた 2 件目の要望。Q-38 を経て D-57 で設計を決め、実装とテストはピコちゃん（opencode、Herdr agent `piko`）が担当し、クロコさんがレビューして AC-37 を測定した。

## 実装の所在

workflow/schema.rs（StateGuard.waived_by と validate）、workflow/store.rs（guard_waiver、current_workflow_issues の免除適用、transition_workflow の waived snapshot）、workflow/history.rs（waived の形の検査）、docs/workflows.md「Waiving a guard with a record」、docs/architecture.md「Workflow policy over time」の 1 段落、README 日英の遷移ガードの節、CHANGELOG、crates/gy/tests/configured_workflow.rs の 2 テストと設定エラーのケース。examples/workflow.toml は変えていない。

## 残った判断

- 版番号: 設定キーと snapshot の任意項目の追加で、書かない台帳の出力は変わらない互換変更。0.4.2。公開はマスターの指示で行う。
- `required = true` の記録は waived_by では免除されない（ガードの records と checks だけが対象）。文書に 1 文足した。
- 免除記録に誰が書けるかは gy が判定しない。承認者を必須項目にするかはプロジェクトの schema が決める。
- Kokopelli 側の残作業: legacy_closure の schema を定義し、complete を含むガードに waived_by を書き、#5969 に記録を書いて advance する。同梱 profile では deviations が object 型の記録なので、core の deviations=none ではなく `{"report":{"status":"none"}}` の形にする。

## 公開

2026-09-14 マスターの指示で 0.4.2 として公開。コミット b15505a（feat）と 823ae5f（chore: release 0.4.2）、タグ v0.4.2 を origin/main へ push。crates.io に gy-core 0.4.2 と gy 0.4.2 を公開した。残るのは Kokopelli への案内のみ。


