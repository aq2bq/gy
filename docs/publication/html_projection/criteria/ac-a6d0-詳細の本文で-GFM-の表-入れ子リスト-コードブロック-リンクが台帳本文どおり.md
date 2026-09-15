# ac-a6d0 (AC-28) 詳細の本文で GFM の表・入れ子リスト・コードブロック・リンクが台帳本文どおりに描かれ、埋め込みエスケープの既存テストが通る

satisfied: true (N-23 / D-42: detail.js の自作レンダラを拡張、外部ライブラリ追加なし。reading fixture D-504 を Markdown preserves table cells, list hierarchy, code and safe links で検証: th 3/td 6、left-center-right揃え、escaped pipeと行内コード内pipe、根ulのli 2/子olのli 2/最大階層3、ol開始番号3、pre 1、複数行コードの改行とtab一致、行内コード/強調/HTTPリンクのhrefとtitle/IDリンク一致。javascript anchor 0、script/img要素0、実行フラグ未定義、U+2028/U+2029のsource保持。スクリーンショットも確認。npm test 28 passed (49.4s、N-21追補テスト含む)、cargo test --workspace --locked 71 passed、fmt/clippy exit 0。ログ /private/tmp/gy-n23-e2e.log、/private/tmp/gy-n23-rust.log。docs/architecture.md HTML projection本文描画節とe2e/README.md reading fixture節を実装・測定内容へ同期。) at 2026-09-13T13:31:07.947860+00:00
scope: html_projection
created: 2026-09-13
  targeted-by n-cda9 (N-23) 詳細本文の Markdown 描画を GFM の表・入れ子リスト・コード・リンクに対応させる



