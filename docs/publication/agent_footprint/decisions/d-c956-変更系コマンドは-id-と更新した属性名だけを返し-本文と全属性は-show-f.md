# d-c956 (D-34) 変更系コマンドは id と更新した属性名だけを返し、本文と全属性は show / find が返す

- 種類: decision
- scope: agent_footprint
- created: 2026-09-13
- 別名: D-34

## 関係

- closed-by q-e7c8 (Q-27) 変更系コマンドの出力から本文を外す範囲をどう定めるか
- narrowed-by d-922b (D-36) 変更系の結果はノード要約 id / type / scope / changed_attributes / body_changed とし、init は解決した root と scope を返す（mark: 本文と全属性は show / find が返す）
- spawns n-adaa (N-17) 変更系コマンドの返却を識別子と更新項目に絞り、本文は show で取得する形にする

## 成立範囲

CLI と MCP の全変更系コマンド（node set、node submit、criterion satisfy、decide、link、need add、question add、req advance など）。出力契約の非互換変更として次の 0.y 版に他の非互換項目とまとめて出す。参照系（show、find、handover、next）の出力は対象外。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

