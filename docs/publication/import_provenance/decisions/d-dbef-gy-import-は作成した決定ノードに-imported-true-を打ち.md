# d-dbef (D-56) gy import は作成した決定ノードに imported: true を打ち、imported かつ成立範囲が空の決定は L14（既定 warn）で報告し、L7 は imported でない決定に限る

- 種類: decision
- scope: import_provenance
- created: 2026-09-14
- 別名: D-56

## 関係

- closed-by q-b51a (Q-37) 取り込み由来で成立範囲が空のままの決定を、L7 とどう区別して報告するか
- spawns n-4534 (N-31) 取り込み時に成立範囲を意図して空欄にした決定の L7 を、新規の欠落と区別して報告する

## 成立範囲

gy import が作成する決定ノードと、決定の decision_scope が空のときの lint 判定。imported の印は取り込み履歴として残り、成立範囲を埋めれば L14 は消える。0.4.0 以前に取り込んだ決定には印が無く、利用者が gy node set <ID> --set imported=true で付ける。L14 は他の規則と同じく [lint] で severity を変えられる。新規の欠落（imported でない決定）は L7 error のまま

## 本文

## Context

Kokopelli の進行担当からマスター経由で受けた要望（N-31）。旧台帳 113 件を `[import] scope_note_placeholders` で意図して成立範囲を空欄のまま取り込んだ結果、L7 が 81 件 error として残り、lint / handover の終了コードが常に 1 になって新しい L5 が 2 日間埋もれた。

## Decision

Q-37 の案1。マスターは推奨案に基づく実装をピコちゃん（opencode）へ委ねる指示で承認した。

- `gy import` は作成した決定ノードすべてに `imported: true` を打つ。辺の `imported` と同じ語で、取り込み履歴を表す。
- lint は `decision_scope` が空の決定を、`imported == true` なら L14（既定 warn）、それ以外は L7（既定 error）で報告する。1 つの決定が両方を出すことはない。
- L14 の文言は L6 に倣い、元の決定を読んで埋めるか移行作業として残すかを促す。
- 選ばなかった案2（L7 に imported 別 severity の設定形）は RuleConfig を L7 専用に複雑にし、L6 との対称性を崩す。案3（L7 全体を warn）は新規の欠落を error に保てず、要望の条件を満たさない。

## Applies to

新規の取り込みと、印を付けた既存の決定。印の無い既存台帳では lint の出力は変わらない。

## 自由属性

- 無し

