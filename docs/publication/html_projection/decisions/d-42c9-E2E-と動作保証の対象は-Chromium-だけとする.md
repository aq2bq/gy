# d-42c9 (D-22) E2E と動作保証の対象は Chromium だけとする

## 関係
- 無し

decision_scope: 利用者はマスターとそのエージェントで、生成物はローカルのファイルである。いま複数ブラウザの動作を保証する動機がなく、開発の速さを優先する。3エンジンでの実行は製品の欠陥を1件も出さず、出たのは WebKit におけるテスト側の座標の食い違い1件だけだった。Playwright の WebKit は Safari ではないため、Safari を保証するなら実機が別に必要である。他者へ生成物を渡す運用に変わったときは再検討する
scope: html_projection
created: 2026-09-13
  closed-by q-fd63 (Q-18) E2E と動作保証の対象ブラウザをいくつにするか

## Context

## Decision

## Consequences


