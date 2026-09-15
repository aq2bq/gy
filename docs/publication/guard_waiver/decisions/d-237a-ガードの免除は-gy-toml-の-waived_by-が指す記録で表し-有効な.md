# d-237a (D-57) ガードの免除は gy.toml の waived_by が指す記録で表し、有効な免除記録を持つ要求にはそのガードの records と checks を掛けず、snapshot に免除の事実を残す

- 種類: decision
- scope: guard_waiver
- created: 2026-09-14
- 別名: D-57

## 関係

- closed-by q-36fa (Q-38) ワークフロー外で閉じた要求を complete へ進める経路をどの形で用意するか
- spawns n-7681 (N-32) ワークフローの外で進んだ要求 (legacy) を、証跡付きで complete へ進められる経路を用意する

## 成立範囲

[workflow.guards.<名前>] に waived_by = "<記録名>" を書いたガードと、その記録名の schema を持つ台帳。免除記録は kinds に requirement を含むプロジェクト定義の記録で、schema に対して有効なときだけ効く。遷移の snapshot には免除記録と waived {ガード: 記録} を保存し、免除したガードの checks は保存しない。現在状態の lint / handover の検査も同じ経路。waived_by の無いガードと、免除記録の無い要求の振る舞いは変わらない。gy は免除記録を誰が書けるかを判定しない

## 本文

## Context

Kokopelli の要望 N-32。ワークフロー導入前に進み、外で閉じた要求 #5969 が complete のガードで止まり、handover の missing と終了コードを占有している。記録を後付けで作る案は実在しない作業の記録になるので Kokopelli が退けている。

## Decision

Q-38 の案1。マスターは 2026-09-14 に「OK」で承認した。

- 免除は現在の方針（gy.toml）が定める経路であり、per-node のフラグでも状態でもない。docs/architecture.md「Workflow policy over time」の「per-node exemption flag を使わない」と整合する。
- 免除の根拠は人が書く記録で、schema に対して検査される。既存の「明示的な不在と例外」（variant_field、design-waived）と同じ型。
- 選ばなかった案2（`--legacy` フラグ）は architecture の契約に反し、gy が知らない `legacy_status` と「記録が無い」ことの推測に依存する。案3（closed 状態）は状態集合の変更で完了境界が 2 つになる。案4（変更なし）は閉じた要求が active に残り続ける。

## Applies to

waived_by を書いたガードと、その記録の schema を持つ台帳。書かないプロジェクトの振る舞いは変わらない。

## 自由属性

- 無し

