# d-f6c0 (D-52) ローカルの既定の E2E は機能 spec だけを走らせ、性能ゲートは CI と明示の入口（test:perf / test:all）で走らせる

- 種類: decision
- scope: agent_footprint
- created: 2026-09-14
- 別名: D-52

## 関係

- closed-by q-657a (Q-35) N-30 の直列性能検査を維持した実測が AC-35 の3分の1目標に届かない場合の完了範囲

## 成立範囲

e2e の実行入口。npm test（ローカル既定）は性能 spec を除いた機能 spec を並列で実行し、反復の検証に使う。npm run test:all は全件、npm run test:perf は性能 spec のみ。CI は従来どおり全件を 1 ワーカー・retries 0 で実行し、成果物も不変。性能検査の実行コスト自体は変えない（Q-35 の案 B は採らない）。need の完了報告は test:all の結果で行う。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

