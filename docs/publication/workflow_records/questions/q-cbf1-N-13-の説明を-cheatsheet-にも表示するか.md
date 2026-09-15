# q-cbf1 (Q-26) N-13 の説明を cheatsheet にも表示するか

- 種類: question
- scope: workflow_records
- created: 2026-09-13
- 状態: closed
- 別名: Q-26

## 関係

- closes d-feb4 (D-30) 記録の説明は handover の出力で公開し、cheatsheet には載せない

## 本文

# N-13 cheatsheet への説明表示 — Q-26、PO 判断待ち

gy の開発担当のデック先生です。N-13 の description と handover JSON は実装・検証済みです。cheatsheet の表示は変更していません。

選択肢 A（推奨）は、gy.toml を説明の正本とし、handover JSON で有効な説明を公開する現状の実装までとすることです。cheatsheet は既に handover で設定を発見する使い方を案内しており、再帰的な schema の長さが操作早見表へ加算されません。記録・項目の意味を読む側は handover の workflow.records を参照します。AC-16 はこの形で満たしています。

選択肢 B は、cheatsheet に記録名・項目パス・説明の設定別一覧を追加することです。操作時に意味も同時に読めますが、配列・variant の項目を漏れなくたどる表示規則が必要になり、大きな profile ほど早見表が長くなります。型にするなら、省略した本文を推測させず、完全なパスと説明を表示する規則を定めます。

PO には A と B の採否をお願いします。追加表示がコンテキスト長への依存を減らす実運用上の根拠はまだないため、A を推奨します。

## 閉じ方

- 決定で閉じた（2026-09-13T09:01:16.732708+00:00）

## 自由属性

- 無し

