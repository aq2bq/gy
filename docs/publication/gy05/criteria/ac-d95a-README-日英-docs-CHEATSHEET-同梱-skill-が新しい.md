# ac-d95a (AC-56) README 日英・docs・CHEATSHEET・同梱 skill が新しい gy の 20 の操作（書き 15、読み 5。D-84）と一つの進め方だけを説明し、消した設定・操作・規則への言及が無い（grep で 0）

- 種類: criterion
- scope: gy05
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-56

## 関係

- targeted-by n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示）
- targeted-by n-271d (N-43) 文書と同梱 skill を新しい gy に合わせて書き直す（README 日英、architecture、CHEATSHEET、skill）

## 本文

## 測り方

README 日英・docs・CHEATSHEET・skills を、消した設定・操作・規則の語で grep して 0 であることと、操作の一覧が gy --help の末端と一致することを見る。

## 充足

- satisfied（2026-09-15 lead。README.md / README.ja.md（各 161 行、12 節同順）、docs/architecture.md（65 行）、docs/migration-0.5.md（67 行）、docs/release.md、crates/gy5/CHEATSHEET.md（38 行）、skills 3 つ（27 / 22 / 24 行）。禁止語（render、workflow.records、guards、waived_by、L1〜L14、req advance、node set、node submit、gate、find、import、parent_issue、stats、compress、q、need file）の grep は 0。lint は「後から走らせる lint は無い」の 1 文のみ。20 の操作が README の一覧にあり gy5 --help と一致。コミット b76707d） 2026-09-15T04:35:32.438190+00:00

## 自由属性

- 無し

