# d-affc (D-5) RecordSchema と FieldSchema と RecordCheck は読み取り専用型とし Default を実装しない

## 関係
- 無し

decision_scope: これら3種に意味のある既定値が存在せず、空の Default は validate で必ず落ちる設定を下流が黙って生成できてしまう。構築手段は serde によるデシリアライズのみとし、その位置づけを CHANGELOG に明記する
scope: html_projection
created: 2026-09-13
  closed-by q-a09e (Q-5) Default を持たない RecordSchema / FieldSchema / RecordCheck に Default を足すか

## Context

## Decision

## Consequences


