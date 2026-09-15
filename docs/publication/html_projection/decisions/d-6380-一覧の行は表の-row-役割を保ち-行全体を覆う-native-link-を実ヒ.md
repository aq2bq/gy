# d-6380 (D-48) 一覧の行は表の row 役割を保ち、行全体を覆う native link を実ヒット対象とする。押せる見た目の検査は要素の計算スタイルではなく実ヒット対象で行う

- 種類: decision
- scope: html_projection
- created: 2026-09-13
- 別名: D-48

## 関係

- closed-by q-4473 (Q-32) 常設表の行リンク検査は実際のヒット対象で行うかtrをlinkへ変更するか
- narrows d-6e4d (D-44) 押せるものは押せる見た目を持ち、押せる見た目のものは押すと何かが起きる（mark: ポインタカーソルを持ち）

## 成立範囲

HTML projection の一覧（D-44 の具体化）。tr は role=row のまま（Chromium の AX で row 25→1、cell 144→0 になる role=link は採らない）。行の全列で elementFromPoint の対象が data-record を持つ a であり、cursor は pointer。tr 自身にも pointer を付ける。E2E は全列の実ヒット対象が native link であることと実マウスでの遷移を検査する。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

