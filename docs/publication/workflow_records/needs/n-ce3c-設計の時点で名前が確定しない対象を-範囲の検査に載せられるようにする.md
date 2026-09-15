# n-ce3c (N-11) 設計の時点で名前が確定しない対象を、範囲の検査に載せられるようにする

- 種類: need
- scope: workflow_records
- created: 2026-09-13
- 状態: closed
- 別名: N-11

## 関係

- targets ac-554a (AC-14) 設計の時点で名前が確定しないファイルを含む設計と実装の報告が範囲の検査を通り、宣言されていないファイルが混ざった場合は落ちる

## 本文

## 出所

kokopelli-recurring-v2 の進行管理担当から 2026-09-13 に報告。Issue #6009 の進行中、Rails の migration は実装時にタイムスタンプが採番されるため、設計提案には `db/migrate/<generated>_add_recurring_v2_enabled_to_tf_recurring_accounts.rb` と書き、実物は `20260912045332_add_...rb` になった。

## 再現

最小の台帳に `design_proposal.files` と `implementation_report.files` の same-set を置いて確認した。

```
error: SameSet check failed: implementation_report.files versus design_proposal.files
exit=2
```

`string_set` は要素を文字列の完全一致で集合にする。パターン照合の概念がないため、subset に変えても一致しない。設定では解けない。

## 判定

採用する。集合一致が確かめているのは「宣言した範囲の外に出ていないか」であり、名前の完全一致はその手段の一つにすぎない。宣言の時点で名前が確定しない対象（migration、生成コード、内容ハッシュを含む成果物）は一般的で、特定のプロジェクトの規則ではない。

検査を弱めるのではなく、検査が成立しない事例を成立させる変更として設計する。既存の same-set / subset の意味は変えない。どちら側がどう確定しない部分を宣言できるかを、実装前に定義する。

## 閉じ方

- 事実で閉じた（workflow の記録と範囲の検査は D-60 / N-35 で消した（d-736e））

## 自由属性

- 無し

