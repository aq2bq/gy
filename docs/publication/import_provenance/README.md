# gy の公開物 — import_provenance

- 生成: 2026-09-15T10:22:54Z
- seq: 158
- scope: import_provenance
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
- [n-4534 (N-31) 取り込み時に成立範囲を意図して空欄にした決定の L7 を、新規の欠落と区別して報告する](needs/n-4534-取り込み時に成立範囲を意図して空欄にした決定の-L7-を-新規の欠落と区別して報.md) — closed
### questions
- [q-b51a (Q-37) 取り込み由来で成立範囲が空のままの決定を、L7 とどう区別して報告するか](questions/q-b51a-取り込み由来で成立範囲が空のままの決定を-L7-とどう区別して報告するか.md) — closed
### decisions
- [d-dbef (D-56) gy import は作成した決定ノードに imported: true を打ち、imported かつ成立範囲が空の決定は L14（既定 warn）で報告し、L7 は imported でない決定に限る](decisions/d-dbef-gy-import-は作成した決定ノードに-imported-true-を打ち.md)
### requirements
- 無し
### criteria
- [ac-add0 (AC-36) 取り込み由来で成立範囲が空のままの決定だけが残る台帳で、gy lint と gy handover の終了コードが 0 になり、gy decide で作った決定の成立範囲の欠落は error のまま残る](criteria/ac-add0-取り込み由来で成立範囲が空のままの決定だけが残る台帳で-gy-lint-と-gy.md) — satisfied

## 履歴

| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |
|---|---|---|---|---|---|---|
| 1 | 2026-09-15 04:49 | lead | ac-add0 (AC-36) 取り込み由来で成立範囲が空のままの決定だけが残る台帳で、gy lint と gy handover の終了コードが 0 になり、gy decide で作った決定の成立範囲の欠落は error のまま残る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-dbef (D-56) gy import は作成した決定ノードに imported: true を打ち、imported かつ成立範囲が空の決定は L14（既定 warn）で報告し、L7 は imported でない決定に限る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-4534 (N-31) 取り込み時に成立範囲を意図して空欄にした決定の L7 を、新規の欠落と区別して報告する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-b51a (Q-37) 取り込み由来で成立範囲が空のままの決定を、L7 とどう区別して報告するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 77 | 2026-09-15 10:21 | lead | n-4534 (N-31) 取り込み時に成立範囲を意図して空欄にした決定の L7 を、新規の欠落と区別して報告する | updated | need close n-4534 | import と L14 は 0.5 で消し、未記録の成立範囲は移行の印と edit --set decision_scope で扱う（d-736e） |
| 120 | 2026-09-15 10:22 | lead | ac-add0 (AC-36) 取り込み由来で成立範囲が空のままの決定だけが残る台帳で、gy lint と gy handover の終了コードが 0 になり、gy decide で作った決定の成立範囲の欠落は error のまま残る | updated | edit ac-add0 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |

## 診断

エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。警告は注意が要る状態で、`handover` に一覧が出る。

errors: 0
warnings: 0
