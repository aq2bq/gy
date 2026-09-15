# n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示）

state: closed
scope: gy05
created: 2026-09-15
acceptance: 2026-09-15 lead が (b) を実施: publication/0.4 に凍結し gy-migrate で 258 ノードを移行、next 12/12 一致、handover は open 0 / ready 12 / 本文が空の AC 55。以後 gy5 で操作
2026-09-15 lead。(a)(b)(c) 完了、コミット 425a2f5。gy 0.5.0、末端 20、cargo package に 0.4 の痕跡 0、cargo publish --workspace --dry-run 成功、テスト 190 件 0 failed。公開（crates.io、push、タグ）はマスターの指示待ち
2026-09-15 14:42 マスターの指示で公開: crates.io に gy-ledger 0.5.0 と gy 0.5.0（gy-core と gy-migrate は publish = false）、main を push（0.4.2 から 38 コミット）、タグ v0.5.0 を push。公開時のコミットは c2b9b97
2026-09-15 15:16 マスターの指示で 0.5.1 を公開: crates.io の gy-ledger と gy、main の push、タグ v0.5.1（27af90c）。内容は waits-on の要求への拡張と移行の belongs-to の写し
  depends-on n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）
  depends-on n-271d (N-43) 文書と同梱 skill を新しい gy に合わせて書き直す（README 日英、architecture、CHEATSHEET、skill）
  depends-on n-5091 (N-71) publish (b): ノードの節（問い 1・3・4）と「この期間の変更」（問い 5）、gy5 の配線、publish-sample.md（N-40 から分割）
  depends-on n-b6f3 (N-72) publish を 1 ページの読み物に直す: 既定はいま判断待ち・未決・注意の件数・期間の変更だけ、ノードの記述は publish <ID>... で指定した分だけ
  spawned-by d-16f9 (D-76) 新しい gy は store → model → ops → views の 4 層で新しい core として書き、物理設計と依存の数の基準を常設のゲートにし、超えたら機能より先に分ける
  targets ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  targets ac-d95a (AC-56) README 日英・docs・CHEATSHEET・同梱 skill が新しい gy の 20 の操作（書き 15、読み 5。D-84）と一つの進め方だけを説明し、消した設定・操作・規則への言及が無い（grep で 0）
## 範囲（2026-09-15 lead）

順序: N-41（両台帳の移行が通る）→ Kokopelli への移行の案内（旧 ID の別名、ref、TEAM_AGENTS.md I1 の書き換え。gy の番号が正）→ N-40（publish）→ N-43（文書）→ このニーズ。crates/gy を新しい CLI（今の gy5）に置き換え、gy-core と gy-migrate を消す（移行は済んでいる前提。Kokopelli が移行を終えるまでは gy-migrate をリリースに含める判断もある）。版は roadmap 未決 3（0.5 か 1.0）をマスターに問う。公開（crates.io、push、タグ）はマスターの指示（AGENTS.md「バージョンと公開」）。

閉じた理由: 事実（2026-09-15 公開済み。crates.io の gy-ledger 0.5.0 と gy 0.5.0、タグ v0.5.0（c2b9b97））

