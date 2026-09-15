# ac-5b53 (AC-24) gy.html#<ID> を開くとそのノードの詳細が開いており、焦点・選択・絞り込み・検索語の変更が hash に反映され、ブラウザの戻るで直前の表示状態に戻る

- 種類: criterion
- scope: html_projection
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-24

## 関係

- targeted-by n-afe9 (N-19) 表示状態を URL の hash に載せ、ID で直接到達でき、戻るが効くようにする

## 本文

## 充足

- satisfied（N-19 / D-38: state.js の locationHash が hash codec を一元化。Playwright URL selects IDs, reports missing IDs, and restores state through history で ID直達、未存在ID、別タブ・reload、選択/焦点経路/半径/検索/種別/スコープ/状態/質問状態/criterion/genealogy/tab の復元、戻る/進む、キー操作で history +1、ズームでhash不変を確認。npm test 22 passed (46.4s)、cargo test --workspace --locked 71 passed、fmt/clippy exit 0。ログ /private/tmp/gy-n19-e2e.log と /private/tmp/gy-n19-rust.log。docs/architecture.md Focus navigation and viewport ownership をURL復元と詳細履歴へ同期。） 2026-09-13T13:11:40.012084+00:00

## 自由属性

- 無し

