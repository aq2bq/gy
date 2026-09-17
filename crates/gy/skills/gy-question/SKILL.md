---
name: gy-question
description: gy に論点を立てて閉じるとき、決定者と選択肢と閉じ方を決めるときに使う。
---

# 論点を扱う

## 立てる

`question add` は決定者（`--decider`）と、互いに異なる選択肢（`--options`）を 2 つ以上求める。誰の合意で閉じるかが決まっていないものは、論点にしない。

## 3 つの閉じ方

`question close --by` は次のいずれかを取る。

- `fact`: 事実が決めた。根拠を `--evidence` に書く。
- `decision`: 決定が決めた。`--decision <D>` で決定を指す。その決定は `decide` で先に作る。
- `non-decision`: 決定を指さずに閉じる。

閉じたら、閉じ方を変えることはできない。

## 迷ったら

`question add` の出力は、先頭に `id: <ID>` を出し、次に打てるコマンド（`question close` と `decide`）を示す。
