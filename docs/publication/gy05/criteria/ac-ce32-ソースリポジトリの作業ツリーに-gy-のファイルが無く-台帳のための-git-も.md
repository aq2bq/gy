# ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない

- 種類: criterion
- scope: gy05
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-44

## 関係

- targeted-by n-0cb7 (N-48) 正本の置き場所（XDG のデータディレクトリ + リポジトリのハッシュ）と形式の版 format の読み書き・判定・移行の入口の枠
- targeted-by n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ
- targeted-by n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示）
- targeted-by n-54fe (N-61) CLI (1a): 新しい gy の CLI クレート gy5 の骨格（clap、GY_ACTOR、--json、-C、終了コード、正本の自動作成）と読み 4 の配線

## 本文

## 測り方

gy 自身の台帳を移行した後に git status を打ち、作業ツリーに変化が無いことを見る。正本は ~/.local/share/gy/<鍵>/ にあり、.gy-dir も台帳の git も無い。

## 充足

- satisfied（2026-09-15 lead。正本は ~/.local/share/gy/<リポジトリのハッシュ>/（location.rs）。gy 自身の台帳を移行しても git status に変化 0 件。リポジトリに置くのは gy.toml だけで .gy-dir も台帳の git も要らない（N-48、N-41）） 2026-09-15T03:51:51.420150+00:00

## 自由属性

- 無し

