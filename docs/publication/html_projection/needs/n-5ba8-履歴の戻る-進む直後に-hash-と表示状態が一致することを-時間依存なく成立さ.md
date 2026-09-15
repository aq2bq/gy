# n-5ba8 (N-28) 履歴の戻る/進む直後に hash と表示状態が一致することを、時間依存なく成立させる

state: closed
scope: html_projection
created: 2026-09-13
  spawned-by d-626e (D-38) 場所は URL が持つ: 表示状態は hash に載り、ブラウザの戻るが効き、ID を貼れば開く
  targets ac-8d4d (AC-33) location.spec の履歴復元テストが単独 10 回連続と全件 3 回連続で成功し、goBack 直後の hash と表示状態の一致が待機に依存しない形で検査される
## 出所

2026-09-14、N-27 の照合中に kuroko の隔離 worktree で観測。cad46c1 で `location.spec.mjs` の「URL selects IDs, reports missing IDs, and restores state through history」が全件実行で 1 回失敗、単独 3 回で 1 回失敗。失敗時は `page.goBack()` 直後の hash の `tab` が期待 `jams` に対し `overview`（1 段戻り切っていない、または popstate の処理が pushState を再発行している）。

## 要求

根拠は D-38（戻るが効く）と D-46（CI の検査が契約）。判定は AC-33。

1. 原因を製品側とテスト側で切り分ける。製品側なら、popstate の処理中に saveLocation が pushState を発行して履歴を汚していないか（復元中は保存を抑止する）を確かめる。テスト側なら、`goBack()` の後に hash の変化を待ってから比較する形にする。両方の可能性を測ってから直す。
2. 直した後、単独 10 回連続と全件 3 回連続で成功することを evidence に書く。
3. 維持条件と完了は N-27 と同じ。

## 完了確認（kuroko、2026-09-14）

deck の報告をコミット 249b5e2 で照合。原因はテスト側（URL 保存の完了前に goBack していた）で、popstate 後の再保存は観測されず製品側の履歴汚染は無い。隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed、bun check exit 0、単独 5/5 passed、全件 44/44 を 2 回連続。受領。

閉じた理由: 事実（migrated: complete）

