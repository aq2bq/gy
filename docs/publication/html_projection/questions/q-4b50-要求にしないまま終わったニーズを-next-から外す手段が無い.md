# q-4b50 (Q-19) 要求にしないまま終わったニーズを next から外す手段が無い

- 種類: question
- scope: html_projection
- created: 2026-09-13
- 状態: closed
- 別名: Q-19

## 関係

- closes d-5ee8 (D-28) ニーズの完了は need 自身の状態が記録する

## 本文

## 観測

gy の開発を gy で記録した結果、`gy next` が完了済みのニーズを返し続けている。

N-4（分割）、N-5（焦点スタック）、N-6（経路）、N-7（E2E）は 0.3.1 で実装と検収を終えたが、`next` は7件すべてを返す。

## 機構

`next` がニーズを除くのは、`filed-as` の先の要求が完了しているときである（`docs/architecture.md` の「`next` stops traversing work prerequisites when it reaches a completed requirement」と、README の「完了要求に対応済みのニーズは除きます」）。

requirement の ID は確認済みの GitHub Issue 番号であり、gy 自身の開発にはその Issue が無い。したがってニーズを要求へ起票する経路が存在せず、完了を記録する手段も無い。

## なぜ問題になりうるか

`next` は「次に何をすべきか」を答えるコマンドである。終わった作業を返し続けると、その答えが実態と合わなくなる。AGENTS.md の「gy が防ぐ失敗」は、終盤で進行管理が崩れることを防ぐと述べており、`next` の過剰報告はその崩れ方の一つに当たる。

いっぽう、これは GitHub Issue を使わない利用者に限る話である。Issue を使う利用者では `need file` と `req advance` で表現できる。gy が担うべき一般的な意味なのか、Issue を使わない運用に残る作業なのかは、「要求を裁く」の基準で判定する必要がある。

## 閉じ方

- 決定で閉じた（2026-09-13T08:54:24.430634+00:00）

## 自由属性

- 無し

