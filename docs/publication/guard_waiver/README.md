# gy の公開物 — guard_waiver

- 生成: 2026-09-15T10:22:53Z
- seq: 158
- scope: guard_waiver
- since: (先頭から)
- 書き手: lead
- 正本: /Users/pememo/.local/share/gy/fae59914

## 読み方

ノードの種類（5 種）:
- need（n-…）: これから行う作業。受け入れ条件を対象にする。
- question（q-…）: まだ決めていないこと。決定者と 2 つ以上の選択肢を持つ。
- decision（d-…）: 条件付きの決定。成立範囲（decision_scope）を持つ。
- requirement（r-…）: ニーズを承認に回したもの。外への参照（ref）を持つ。
- criterion（ac-…）: 受け入れ条件。作業の進みを数える相手。

要求の状態（4 つ）:
- filed: 起票済み。設計はまだ承認されていない。
- approved: 確定。マスターの設計承認がある。
- done: 完了。出荷の根拠が記録されている。
- cancelled: 中止。完了以外の理由で閉じた。

関係（12）と向き:
- closes: 論点 → 決定。
- narrows / widens / supersedes / completes: 決定 → 決定。
- targets: ニーズ / 要求 → 受け入れ条件。
- spawned-by: ニーズ → 決定。
- filed-as: ニーズ → 要求。
- depends-on: ニーズ → ニーズ。
- relies-on: 要求 → 決定。
- raised: 要求 → 論点。
- waits-on: ニーズ → 論点・要求。
辺は始点のノードだけに保存し、逆向きは導出する。この文書は両向きを出す。

論点の閉じ方:
- fact: 事実で閉じた。 - decision: 決定で閉じた。 - non-decision: 決定を伴わずに閉じた。

ID と別名:
- ID は種類の接頭辞と短いハッシュ（例 n-3f9a）。旧 ID があれば別名として併記する（例 D-78）。
- 参照はすべて「ID（別名） 題名」で書く。要求は ref（外への参照）も持つ。

履歴の項目:
- seq: 書き込みの連番（時点）。 - 日時・書き手: いつ・誰が（GY_ACTOR）。
- ノード: 対象の ID と題名。 - 何を: created / updated / deleted。
- なぜ: 操作の名前と対象。 - 出典: 操作に渡した根拠や URL。

## 一覧

### needs
- [n-7681 (N-32) ワークフローの外で進んだ要求 (legacy) を、証跡付きで complete へ進められる経路を用意する](needs/n-7681-ワークフローの外で進んだ要求-legacy-を-証跡付きで-complete-へ.md) — closed
### questions
- [q-36fa (Q-38) ワークフロー外で閉じた要求を complete へ進める経路をどの形で用意するか](questions/q-36fa-ワークフロー外で閉じた要求を-complete-へ進める経路をどの形で用意するか.md) — closed
### decisions
- [d-237a (D-57) ガードの免除は gy.toml の waived_by が指す記録で表し、有効な免除記録を持つ要求にはそのガードの records と checks を掛けず、snapshot に免除の事実を残す](decisions/d-237a-ガードの免除は-gy-toml-の-waived_by-が指す記録で表し-有効な.md)
### requirements
- 無し
### criteria
- [ac-7145 (AC-37) ワークフローの外で閉じた要求を、実在しない作業記録を作らずに証跡付きで complete へ進められ、ワークフローを通る要求の complete のガードは変わらず、完了後の lint / handover が履歴を再検査しても指摘を出さない](criteria/ac-7145-ワークフローの外で閉じた要求を-実在しない作業記録を作らずに証跡付きで-comp.md) — satisfied

## 履歴

| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |
|---|---|---|---|---|---|---|
| 1 | 2026-09-15 04:49 | lead | ac-7145 (AC-37) ワークフローの外で閉じた要求を、実在しない作業記録を作らずに証跡付きで complete へ進められ、ワークフローを通る要求の complete のガードは変わらず、完了後の lint / handover が履歴を再検査しても指摘を出さない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-237a (D-57) ガードの免除は gy.toml の waived_by が指す記録で表し、有効な免除記録を持つ要求にはそのガードの records と checks を掛けず、snapshot に免除の事実を残す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-7681 (N-32) ワークフローの外で進んだ要求 (legacy) を、証跡付きで complete へ進められる経路を用意する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-36fa (Q-38) ワークフロー外で閉じた要求を complete へ進める経路をどの形で用意するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 74 | 2026-09-15 10:21 | lead | n-7681 (N-32) ワークフローの外で進んだ要求 (legacy) を、証跡付きで complete へ進められる経路を用意する | updated | need close n-7681 | 遷移ガードと免除は D-60 / N-35 で消した（d-736e） |
| 106 | 2026-09-15 10:22 | lead | ac-7145 (AC-37) ワークフローの外で閉じた要求を、実在しない作業記録を作らずに証跡付きで complete へ進められ、ワークフローを通る要求の complete のガードは変わらず、完了後の lint / handover が履歴を再検査しても指摘を出さない | updated | edit ac-7145 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |

## 診断

エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。警告は注意が要る状態で、`handover` に一覧が出る。

errors: 0
warnings: 0
