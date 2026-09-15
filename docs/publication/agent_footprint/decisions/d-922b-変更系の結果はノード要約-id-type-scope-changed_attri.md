# d-922b (D-36) 変更系の結果はノード要約 id / type / scope / changed_attributes / body_changed とし、init は解決した root と scope を返す

## 関係
- narrows d-c956 (D-34) 変更系コマンドは id と更新した属性名だけを返し、本文と全属性は show / find が返す（mark: 本文と全属性は show / find が返す）

decision_scope: N-17 の全変更系コマンド（CLI と MCP 共通）。changed_attributes は保存前後で値が変わった属性名（作成時は全属性名）、body_changed は本文が変わったかの真偽値、同値の再設定は空配列と false。属性の値と本文は返さない。req compress の evidence なしは参照操作として既存の全文プレビューを維持し、evidence ありは要約を返す。D-34 を具体化する決定であり、判定の根拠は「結果に残るのは操作が新たに生んだ情報」。
scope: agent_footprint
created: 2026-09-13
  narrows d-c956 (D-34) 変更系コマンドは id と更新した属性名だけを返し、本文と全属性は show / find が返す（mark: 本文と全属性は show / find が返す）
  closed-by q-c0fd (Q-29) N-17 の本文のみの更新とノードを持たない init の結果を、識別子・属性名だけの契約でどう表すか

## Context

## Decision

## Consequences


