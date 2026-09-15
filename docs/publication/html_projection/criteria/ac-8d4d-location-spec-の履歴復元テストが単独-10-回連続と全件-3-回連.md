# ac-8d4d (AC-33) location.spec の履歴復元テストが単独 10 回連続と全件 3 回連続で成功し、goBack 直後の hash と表示状態の一致が待機に依存しない形で検査される

- 種類: criterion
- scope: html_projection
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-33

## 関係

- targeted-by n-5ba8 (N-28) 履歴の戻る/進む直後に hash と表示状態が一致することを、時間依存なく成立させる

## 本文

## 測り方

0.4 系の受け入れ条件。対象のニーズは D-80 / d-736e で閉じた（消した機能か、新しい gy が別の形で満たした）。測り方は当時の satisfy の evidence に記録がある。

## 充足

- satisfied（{"commit": "249b5e2", "diagnosis": {"baseline": "de0212a", "instrumented_runs": 10, "failures": 2, "result": "Click scheduled URL save; traversal began before entry persistence. Both failing traces had pushState before popstate and no pushState after popstate. Test-side ordering race.", "artifact": "/private/tmp/gy-n28-diagnose-results.json"}, "isolated": {"runs": 10, "passed": 10, "log": "/private/tmp/gy-n28-ten.log"}, "full": [{"run": 1, "exit": 0, "seconds": 96.131, "log": "/private/tmp/gy-n28-full-1.log"}, {"run": 2, "exit": 0, "seconds": 97.355, "log": "/private/tmp/gy-n28-full-2.log"}, {"run": 3, "exit": 0, "seconds": 95.603, "log": "/private/tmp/gy-n28-full-3.log"}], "full_passed_each": 44, "checks": {"fmt": "passed", "clippy": "passed", "rust_workspace": 71, "bun_check": "passed", "diff_check": "passed"}, "assertions": "Exact Back/Forward destination URL, matching selected tab, unchanged history length; wait for URL persistence before Back; no fixed sleeps.", "documentation": "e2e/README.md: history checks paragraph added, matched to location.spec.mjs persistence/restoration assertions.", "migration": "none", "publication": "not pushed or published"}） 2026-09-13T23:37:52.297878+00:00

## 自由属性

- 無し

