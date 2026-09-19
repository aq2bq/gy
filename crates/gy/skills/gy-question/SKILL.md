---
name: gy-question
description: gy に論点を立てて閉じるとき、決定者と選択肢と閉じ方を決めるときに使う。
---

# 論点を扱う

## 立てる

`question add` は決定者（`--decider`）と、互いに異なる選択肢を 2 つ以上求める（`--options A --options B` と繰り返す）。ニーズが待つ論点はニーズについて、要求が生んだ論点は要求について問う（`gy-loop` の「進める」）。誰の合意で閉じるかが決まっていないものは、論点にしない。本文は選択肢の根拠なので、`--body-file` で作成と一緒に書く。

論点は何のためかを辺で持つ。それを待つニーズがあれば `link <need> waits-on <q>` で繋ぐ（要求が生んだなら `raised`）。繋がるまで出力の `missing` に「待つニーズ（waits-on）か生んだ要求（raised）」が出て、`handover` は誰も待っていない開いた論点を数える。ニーズは必須ではない。論点 → 決定 → `spawned-by` でニーズ、の順でもよい。

## 3 つの閉じ方

`question close --by` は次のいずれかを取る。

- `fact`: 事実が決めた。根拠を `--evidence` に書く。
- `decision`: 決定が決めた。`--decision <D>` で決定を指す。その決定は `decide` で先に作る。
- `non-decision`: 決定を指さずに閉じる。

閉じたら、閉じ方を変えることはできない。

## 迷ったら

`question add` の出力は、先頭に `id: <ID>` を出し、次に打てるコマンド（`question close`、`decide`、`link … waits-on`）を示す。
