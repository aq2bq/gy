# ac-b77a (AC-26) 検索欄と種別・スコープ・状態の絞り込みがヘッダー直下に常に見え、左パネルは Overview / Blockers / Progress だけになる

satisfied: true (N-21 / D-41: ヘッダー直下の常設toolbarに検索/種別/スコープ/状態/質問状態/criterion/reset/半径/genealogyを配置。左タブはOverview/Progress/Blockersの3個。permanent toolbar remains operable at 1280px/1440px/1920px の3テストで全操作の矩形が画面内、中心のelementFromPointが対象自身/子要素、各タブでも可視を確認。半径2の明示適用・URL再読込復元、Overviewからの状態絞り込みでタブ不変。npm test 26 passed (53.9s)、Rust 71 passed、fmt/clippy exit 0。1280pxのスクリーンショットも目視確認。ログ /private/tmp/gy-n21-e2e.log と /private/tmp/gy-n21-rust.log。docs/architecture.md の焦点操作節をtoolbar/半径/タブの実装へ同期。) at 2026-09-13T13:19:23.398067+00:00
scope: html_projection
created: 2026-09-13
  targeted-by n-ac8e (N-21) 検索と絞り込みをヘッダー直下の常設ツールバーにし、左パネルを状況把握に限る



