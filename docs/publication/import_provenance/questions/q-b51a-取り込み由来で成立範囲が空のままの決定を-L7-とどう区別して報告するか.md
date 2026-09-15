# q-b51a (Q-37) 取り込み由来で成立範囲が空のままの決定を、L7 とどう区別して報告するか

state: closed
decider: master
options: import が決定ノードに imported: true を打ち、imported かつ成立範囲が空の決定を新規則 L14（既定 warn）で報告し、L7 は imported でない決定に限る, 同じ印を使い、規則は L7 のまま、[lint] L7 に imported な決定だけ severity を下げる設定を足す, 印も規則も足さず、利用者が [lint] L7 = warn にする（新規の欠落も warn になる）
scope: import_provenance
created: 2026-09-14
  closes d-dbef (D-56) gy import は作成した決定ノードに imported: true を打ち、imported かつ成立範囲が空の決定は L14（既定 warn）で報告し、L7 は imported でない決定に限る
## Context

Kokopelli の進行担当（クロコさん）からマスター経由で受けた要望（2026-09-14、gy 0.4.0）。旧台帳の決定 113 件を `gy import` で取り込み、成立範囲の欄が旧台帳に無かったため `[import] scope_note_placeholders` で意図して空欄のまま入れた。残る 81 件は元の決定者の意図が要り、まとめて埋めると推論が決定に混ざるため、マスターが埋めない判断をしている。D-113 以降の `gy decide` による決定に L7 は無い。

実害は `gy lint` と `gy handover` の終了コードが常に 1 になり、新しい指摘が 81 件に埋もれること。L5 が 2 日間気づかれなかった。

## 現状の実装

- `gy import` は narrows/supersedes の辺にだけ `imported: true` を打つ（`operations.rs`）。決定ノードには印を打たず、placeholder に一致して空欄のまま置いたという事実は取り込み後に残らない。
- L7 は `decision_scope` が空なら一律 error（`graph.rs`）。L6 は `imported` な辺を文言で区別し、severity は同じ（既定 warn）。
- `lint` / `handover` の終了コードは severity が error の診断が 1 件でもあれば 1。

## 判断の軸

| 軸 | 案1 L14 新設 | 案2 L7 の imported 別 severity | 案3 L7 全体を warn |
| --- | --- | --- | --- |
| 新規の欠落を error に保つ | 保つ | 保つ | 保てない（要望の条件を満たさない） |
| 追加する契約 | 決定ノードの `imported` 属性、規則番号 1 つ、既定 warn | 同じ属性、RuleConfig に L7 専用の形 | なし（今日でも可能） |
| L6 との対称性 | 印は同じ語、severity の既定も L6 と同じ warn | 文言のみ区別する L6 とは別の形 | なし |
| 既存データへの適用 | 利用者が対象の決定に `gy node set <ID> --set imported=true` | 同じ | 不要 |
| 出力の変化 | 印の無い既存台帳では変化なし。互換 | 同じ | なし |

## 推奨

案1。印は辺と同じ語 `imported` を決定ノードにも使い、import が作った決定すべてに打つ。L14 の文言は L6 に倣い「取り込み時に成立範囲が記録されておらず、埋めるには元の決定を読む必要がある」と書く。決定を記録する側で成立範囲を埋めれば L14 は消え、`imported` は取り込み履歴として残る。

閉じ方: 決定（2026-09-14T02:37:22.345886+00:00）

