# n-f2e8 (N-13) 記録の各項目が何を書く場所かを、機械が読める場所に持たせる

- 種類: need
- scope: workflow_records
- created: 2026-09-13
- 状態: closed
- 別名: N-13

## 関係

- targets ac-4305 (AC-16) workflow の各記録と各項目に説明を書け、その説明が handover --json で読める

## 本文

## 出所

kokopelli-recurring-v2 の進行管理担当から 2026-09-13 に報告。`dispatch.synchronization` について定義されているのは型（文字列の配列）だけで、cheatsheet にも handover --json にも意味の記述がない。要求定義担当と進行管理担当が同じ読みをしたが、根拠はなかった。

## 再現

`gy.toml` の項目に説明を書こうとすると、台帳全体が読めなくなる。

```
error: gy.toml: TOML parse error
unknown field `description`, expected one of `type`, `required`, `allow_empty`, `values`, `equals`, `fields`, `items`, `variant_field`, `variants`
exit=3
```

`FieldSchema` と `RecordSchema` は `deny_unknown_fields` である。プロジェクトが項目の意味を書く場所が塞がれている。

## 判定

採用する。意味の内容はプロジェクト固有の規則であり gy は定義しないが、意味を書く場所は gy の責務である。場所がなければプロジェクトは規則を書けない。

説明は検査に使わない。保存して derived output に載せるだけで、gy が意味を解釈することはない。`handover --json` は workflow 設定を丸ごと出しているので、schema に項目を足せばそのまま機械が読める場所に載る。cheatsheet への露出は別途判断する。

## 閉じ方

- 事実で閉じた（workflow の記録の型は D-60 / N-35 で消した（d-736e））

## 自由属性

- 無し

