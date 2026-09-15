# q-36fa (Q-38) ワークフロー外で閉じた要求を complete へ進める経路をどの形で用意するか

- 種類: question
- scope: guard_waiver
- created: 2026-09-14
- 状態: closed
- 別名: Q-38

## 関係

- closes d-237a (D-57) ガードの免除は gy.toml の waived_by が指す記録で表し、有効な免除記録を持つ要求にはそのガードの records と checks を掛けず、snapshot に免除の事実を残す

## 本文

## Context

Kokopelli の進行担当からマスター経由で受けた 2 件目の要望（2026-09-14、gy 0.4.1）。移行時に legacy_status「進行中（状態集合の外）」で入れた要求 #5969 は status が defining、transitions も workflow records も無い。実態は閉じている（GitHub は 2026-09-08 にクローズ、挙げた論点 Q-17 / Q-18 も閉じている）が、`gy req advance 5969 --to complete` は complete に掛かる approval / audit / delivery のガードで止まる。記録を後付けで作れば通るが実在しない作業の記録になるので採らない、という判断を Kokopelli が持っている。残る実害は handover の missing に 1 件残り終了コードが 1 のままなこと。

## 現状の設計契約

- ガードは `[workflow.guards.<名前>]` の `states` に列挙した宛先状態すべてに、前の状態に関わらず掛かる（docs/workflows.md）。宛先が complete なら complete を含む全ガードの records と checks を満たす必要がある。
- docs/architecture.md「Workflow policy over time」: 新しい遷移は常に現在の宛先ガードで評価し、受動検査（履歴）と現在の操作の検証を分ける。その区別は「per-node exemption flag, import marker, timestamp guess, compression status」ではなく経路の分離で表す。
- 記録の型にはすでに「明示的な不在と例外」の機構がある（variant_field による `none` / `not-applicable` / `design-waived`）。免除は理由を伴う記録として書く、が既存の型。
- gy は `legacy_status` を知らない。Kokopelli 側の属性である。
- 遷移の snapshot は宛先ガードの checks を保存し、lint は履歴を保存時の schema と checks で再検査する（history.rs）。ガードを飛ばした遷移で checks をそのまま保存すると、完了後の lint が履歴で失敗する。

## 判断の軸

| 軸 | 案1 waived_by 記録 | 案2 --legacy | 案3 closed 状態 | 案4 変更なし |
| --- | --- | --- | --- | --- |
| ワークフローを通る要求の complete ガード | 変わらない | 変わらない | 変わらない | 変わらない |
| 免除の根拠を誰が何として記録するか | 人がプロジェクト定義の記録（理由・外部証跡・承認者）として書き、schema で検査される | 遷移時のフラグ。根拠は evidence 文字列だけ | 状態そのもの。根拠は evidence 文字列だけ | next_evidence / responsible |
| architecture の「per-node exemption flag を使わない」との整合 | 整合する。免除は現在の方針（gy.toml）が定める経路 | 反する | 状態集合の変更で、方針の外に出口を作る | 整合 |
| gy が知らない属性への依存 | なし | legacy_status と「記録が無い」ことの推測 | legacy_status | なし |
| 追加する契約 | ガード設定に 1 キー、snapshot に免除の項目 | CLI フラグ、transitions の項目 | 状態集合（L10、完了境界、next、handover、compression、HTML、skills、文書） | なし |
| 完了後の履歴再検査 | 免除したガードの checks を保存せず、免除記録を保存するので通る | 飛ばした事実を別途保存する設計が要る | complete ではないので履歴検査の対象外 | 該当なし |
| 利用者側の作業 | 免除記録の schema を定義し、対象ガードに waived_by を書く。#5969 に記録を書いて advance | フラグ付きで advance | advance | 2 属性を書く |
| 残る歪み | なし | なし | 完了境界が 2 つになる | 実態が閉じた要求が active に残り続ける |

## 推奨

案1。免除は「明示的な不在」の既存の型に沿う記録であり、方針は gy.toml に置かれ、gy は記録の整合だけを見る。設定例:

```toml
[workflow.records.legacy_closure]
kinds = ["requirement"]
fields.reason = { type = "string" }
fields.evidence = { type = "url" }
fields.closed_on = { type = "string" }
fields.approver = { type = "string" }

[workflow.guards.audit]
states = ["awaiting-audit", "...", "complete"]
records = ["dispatch", "design_proposal", "approval", "implementation_report", "quality_gates", "deviations"]
waived_by = "legacy_closure"
```

有効な `legacy_closure` を持つ要求には audit ガードの records と checks が掛からない。遷移の snapshot には `legacy_closure` と `waived: {"audit": "legacy_closure"}` が残り、免除したガードの checks は保存しない。lint と handover の現在状態の検査も同じ経路を使う。waived_by を書かないプロジェクトには出口が無い。免除記録に誰が書けるかを gy は判定しないので、承認者を必須項目にするかはプロジェクトが決める。

## 閉じ方

- 決定で閉じた（2026-09-14T03:13:04.069891+00:00）

## 自由属性

- 無し

