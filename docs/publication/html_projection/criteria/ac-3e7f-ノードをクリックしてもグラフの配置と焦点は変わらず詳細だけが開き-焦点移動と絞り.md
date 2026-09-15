# ac-3e7f (AC-25) ノードをクリックしてもグラフの配置と焦点は変わらず詳細だけが開き、焦点移動と絞り込みは押す前に結果が読める明示ボタンで行われる

- 種類: criterion
- scope: html_projection
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-25

## 関係

- targeted-by n-f025 (N-20) ノードのクリックを詳細表示だけにし、焦点移動と絞り込みを明示操作にする

## 本文

## 測り方

0.4 系の受け入れ条件。対象のニーズは D-80 / d-736e で閉じた（消した機能か、新しい gy が別の形で満たした）。測り方は当時の satisfy の evidence に記録がある。

## 充足

- satisfied（N-20 / D-39: グラフ・キーボード・関係リンク・クラスタメンバー・Blockers の選択を詳細表示に統一。詳細領域を常設し、選択時の graph transform と全ノード transform が前後一致（node selection preserves placement, focus and filters until explicit focus）。関係選択で fitChanges=0、明示焦点移動で1。ボタンは対象ID/半径/フィルタ前件数/描画上限を表示し、焦点経路を維持。npm test 23 passed (51.4s)、cargo test --workspace --locked 71 passed、fmt/clippy exit 0。ログ /private/tmp/gy-n20-e2e.log、/private/tmp/gy-n20-rust.log。docs/architecture.md の個別描画・焦点操作の説明を同期。） 2026-09-13T13:15:24.868702+00:00

## 自由属性

- 無し

