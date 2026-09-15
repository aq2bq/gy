# gy の公開物 — need_lifecycle

- 生成: 2026-09-15T10:22:54Z
- seq: 158
- scope: need_lifecycle
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
- [n-44d8 (N-14) ニーズのライフサイクルを設計として定義し、文書と同梱スキルに載せる](needs/n-44d8-ニーズのライフサイクルを設計として定義し-文書と同梱スキルに載せる.md) — closed
- [n-fe2d (N-15) 定義された状態のいずれでもないニーズの状態を lint が検出する](needs/n-fe2d-定義された状態のいずれでもないニーズの状態を-lint-が検出する.md) — closed
### questions
- 無し
### decisions
- [d-5ee8 (D-28) ニーズの完了は need 自身の状態が記録する](decisions/d-5ee8-ニーズの完了は-need-自身の状態が記録する.md)
### requirements
- 無し
### criteria
- [ac-736c (AC-18) ニーズの status が定義された状態のいずれでもない値のとき lint が検出する](criteria/ac-736c-ニーズの-status-が定義された状態のいずれでもない値のとき-lint-が検.md) — unsatisfied
- [ac-aef7 (AC-17) ニーズの完了の記録方法と next の判定が README・cheatsheet・同梱スキル・architecture.md に書かれている](criteria/ac-aef7-ニーズの完了の記録方法と-next-の判定が-README-cheatsheet.md) — unsatisfied

## 履歴

| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |
|---|---|---|---|---|---|---|
| 1 | 2026-09-15 04:49 | lead | ac-aef7 (AC-17) ニーズの完了の記録方法と next の判定が README・cheatsheet・同梱スキル・architecture.md に書かれている | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-736c (AC-18) ニーズの status が定義された状態のいずれでもない値のとき lint が検出する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-5ee8 (D-28) ニーズの完了は need 自身の状態が記録する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-44d8 (N-14) ニーズのライフサイクルを設計として定義し、文書と同梱スキルに載せる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-fe2d (N-15) 定義された状態のいずれでもないニーズの状態を lint が検出する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 76 | 2026-09-15 10:21 | lead | n-fe2d (N-15) 定義された状態のいずれでもないニーズの状態を lint が検出する | updated | need close n-fe2d | lint は D-75 で無くし、ニーズの状態は導出（open / closed / done）（d-736e） |
| 78 | 2026-09-15 10:21 | lead | n-44d8 (N-14) ニーズのライフサイクルを設計として定義し、文書と同梱スキルに載せる | updated | need close n-44d8 | ニーズのライフサイクルは gy 0.5 の導出（need_state）と README・skill で定義した（d-736e） |
| 107 | 2026-09-15 10:22 | lead | ac-736c (AC-18) ニーズの status が定義された状態のいずれでもない値のとき lint が検出する | updated | edit ac-736c | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 121 | 2026-09-15 10:22 | lead | ac-aef7 (AC-17) ニーズの完了の記録方法と next の判定が README・cheatsheet・同梱スキル・architecture.md に書かれている | updated | edit ac-aef7 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |

## 診断

エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。警告は注意が要る状態で、`handover` に一覧が出る。

errors: 0
warnings: 0
