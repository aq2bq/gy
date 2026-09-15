# gy の公開物 — workflow_records

- 生成: 2026-09-15T07:59:26Z
- seq: 45
- scope: workflow_records
- since: (先頭から)
- 書き手: piko
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
- [n-54b9 (N-12) 母数の概念が無いゲートを、数値を作らずに記録できるようにする](needs/n-54b9-母数の概念が無いゲートを-数値を作らずに記録できるようにする.md) — open
- [n-ce3c (N-11) 設計の時点で名前が確定しない対象を、範囲の検査に載せられるようにする](needs/n-ce3c-設計の時点で名前が確定しない対象を-範囲の検査に載せられるようにする.md) — open
- [n-f2e8 (N-13) 記録の各項目が何を書く場所かを、機械が読める場所に持たせる](needs/n-f2e8-記録の各項目が何を書く場所かを-機械が読める場所に持たせる.md) — open
### questions
- [q-321b (Q-23) 内容が同じと検算できる訂正のとき、設計の版を上げても承認を引き継げるようにするか](questions/q-321b-内容が同じと検算できる訂正のとき-設計の版を上げても承認を引き継げるようにするか.md) — closed
- [q-3d7a (Q-24) プロジェクトの受け入れ条件に紐づかない作業を、そのプロジェクトの台帳に置くべきか](questions/q-3d7a-プロジェクトの受け入れ条件に紐づかない作業を-そのプロジェクトの台帳に置くべきか.md) — closed
- [q-cbf1 (Q-26) N-13 の説明を cheatsheet にも表示するか](questions/q-cbf1-N-13-の説明を-cheatsheet-にも表示するか.md) — closed
- [q-f122 (Q-25) N-11 の未確定ファイル名は汎用 glob と生成部分の型付き宣言のどちらで範囲照合するか](questions/q-f122-N-11-の未確定ファイル名は汎用-glob-と生成部分の型付き宣言のどちらで範.md) — closed
### decisions
- [d-54ec (D-27) 受け入れ条件を立てられない作業は、その台帳の対象ではない](decisions/d-54ec-受け入れ条件を立てられない作業は-その台帳の対象ではない.md)
- [d-d069 (D-29) 未確定のファイル名は、生成部分の文字種と長さを型として宣言する](decisions/d-d069-未確定のファイル名は-生成部分の文字種と長さを型として宣言する.md)
- [d-d224 (D-31) 設計の版を上げた訂正で承認を引き継がない](decisions/d-d224-設計の版を上げた訂正で承認を引き継がない.md)
- [d-feb4 (D-30) 記録の説明は handover の出力で公開し、cheatsheet には載せない](decisions/d-feb4-記録の説明は-handover-の出力で公開し-cheatsheet-には載せな.md)
### requirements
- 無し
### criteria
- [ac-1b8a (AC-15) 母数の概念が無いゲートの合格を、数値を作らずに記録でき、母数のあるゲートでは母数の欠落が検出される](criteria/ac-1b8a-母数の概念が無いゲートの合格を-数値を作らずに記録でき-母数のあるゲートでは母数.md) — satisfied
- [ac-4305 (AC-16) workflow の各記録と各項目に説明を書け、その説明が handover --json で読める](criteria/ac-4305-workflow-の各記録と各項目に説明を書け-その説明が-handover.md) — satisfied
- [ac-554a (AC-14) 設計の時点で名前が確定しないファイルを含む設計と実装の報告が範囲の検査を通り、宣言されていないファイルが混ざった場合は落ちる](criteria/ac-554a-設計の時点で名前が確定しないファイルを含む設計と実装の報告が範囲の検査を通り-宣.md) — satisfied

## 履歴

| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |
|---|---|---|---|---|---|---|
| 1 | 2026-09-15 04:49 | lead | ac-554a (AC-14) 設計の時点で名前が確定しないファイルを含む設計と実装の報告が範囲の検査を通り、宣言されていないファイルが混ざった場合は落ちる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-1b8a (AC-15) 母数の概念が無いゲートの合格を、数値を作らずに記録でき、母数のあるゲートでは母数の欠落が検出される | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-4305 (AC-16) workflow の各記録と各項目に説明を書け、その説明が handover --json で読める | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-54ec (D-27) 受け入れ条件を立てられない作業は、その台帳の対象ではない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-d069 (D-29) 未確定のファイル名は、生成部分の文字種と長さを型として宣言する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-feb4 (D-30) 記録の説明は handover の出力で公開し、cheatsheet には載せない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-d224 (D-31) 設計の版を上げた訂正で承認を引き継がない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-ce3c (N-11) 設計の時点で名前が確定しない対象を、範囲の検査に載せられるようにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-54b9 (N-12) 母数の概念が無いゲートを、数値を作らずに記録できるようにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-f2e8 (N-13) 記録の各項目が何を書く場所かを、機械が読める場所に持たせる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-321b (Q-23) 内容が同じと検算できる訂正のとき、設計の版を上げても承認を引き継げるようにするか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-3d7a (Q-24) プロジェクトの受け入れ条件に紐づかない作業を、そのプロジェクトの台帳に置くべきか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-f122 (Q-25) N-11 の未確定ファイル名は汎用 glob と生成部分の型付き宣言のどちらで範囲照合するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-cbf1 (Q-26) N-13 の説明を cheatsheet にも表示するか | created | migrate from 0.4 | publication/0.4 0d42e55 |

## 診断

エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。警告は注意が要る状態で、`handover` に一覧が出る。

errors: 0
warnings: 1
- criteria with an empty body: 3
