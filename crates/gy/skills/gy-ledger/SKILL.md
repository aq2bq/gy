---
name: gy-ledger
description: gy の台帳のノードと辺を操作するとき、ニーズ・要求・受け入れ条件の繋ぎ方と閉じ方、チームで共有する台帳の振る舞いを確かめるときに使う。仕事の進め方は gy-loop。
---

# gy の台帳

仕事の進め方（復元する、1 つ選ぶ、進める、止まる、満たす）は `gy-loop` にある。ここはノードと辺の意味である。コマンドの形は `gy-loop` の `CHEATSHEET.md` にある。

## ニーズ・要求・受け入れ条件

- ニーズは受け入れ条件を `targets` で指す。`need add` は出力の先頭に `id: <ID>` を出し、`next` が次のコマンドを示す。本文は `--body-file` で作成と一緒に入る。
- 要求は `req add` で立て、担うニーズ（`--need`）、使える決定（`--relies-on`）、受け入れ条件（`--targets`）に結ぶ。
- 着手の前提は `depends-on` と `waits-on` の辺で表す。`next` はこれを見る。
- ニーズは、`filed-as` の要求が完了すれば導出で `done` になる。`need close` で閉じても受け入れ条件は満たされない。閉じたときの `missing` に未達の条件が出るので、証拠があれば `criterion satisfy`、取り下げるなら `link --remove <need> targets <ac>`。
- 日付は自分の場所の時刻で出る。他の書き手に時点を伝えるときは日付でなく `seq` か ID を使う。

## チームの台帳（gy.toml に remote があるとき）

- 書きは今までどおり。push は背景で行われ、`gy sync` で明示にもできる。`handover` の先頭に未 push の件数と最終同期が出る。
- 標準エラーに `notice: your write seq … did not land` が出たら、その書きは相手が先に同じノードを変えたため載らなかった。今の台帳を読み直し、まだ要るならやり直す。
- `undo` は自分の書きだけ。
