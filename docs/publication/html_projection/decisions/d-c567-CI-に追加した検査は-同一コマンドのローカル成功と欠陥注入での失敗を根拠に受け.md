# d-c567 (D-46) CI に追加した検査は、同一コマンドのローカル成功と欠陥注入での失敗を根拠に受け入れ、GitHub 上の実行結果は push 後に追補の evidence として確かめる

## 関係
- 無し

decision_scope: gy の開発で、push がマスターの判断待ちのまま受け入れを判定する場合。ローカルで CI と同一のコマンドが成功し、注入した不一致を検出することを根拠とする。push 後に GitHub の実行結果を followup_evidence に記録する義務が残る。
scope: html_projection
created: 2026-09-13
  spawns n-9ed7 (N-30) E2E の 1 回の実行を軽くし、fixture の再生成をソースの変化がある時だけにする
  closed-by q-6838 (Q-30) N-25 の CI 再現検査はローカル実行の結果で受け入れるか、GitHub 実行まで待つか

## Context

## Decision

## Consequences


