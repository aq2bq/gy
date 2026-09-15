# ac-9be9 (AC-40) gy.toml に書けるのはスコープと出力先だけで、他のキーは読み込み時にエラー。Kokopelli の gy.toml 2,117 行が 16 行相当で読み込める

- 種類: criterion
- scope: gy05
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-40

## 関係

- targeted-by n-3d8c (N-35) workflow.records / guards / スナップショット / lint 設定 / import 設定 / 申告オプション / gate を消し、gy.toml をスコープと出力先だけにする
- targeted-by n-d1bf (N-50) 操作の共通の枠: gy.toml（スコープと出力先だけ）の読み込み、型付きノードの読み書きと ID・別名の解決を担う Repository、1 意図 1 トランザクションの実行の型、出力の型（ID・変えた項目・無いもの・次の操作）

## 本文

## 充足

- satisfied（2026-09-15 N-50。gy.toml に書けるのは [scopes.<名前>] と output だけで deny_unknown_fields。Kokopelli の gy.toml の写し 2,117 行を読ませると unknown field parent_issue で Err（tests/config.rs）。移行が生成する gy.toml は [scopes.select_bin_v2] の 1 行） 2026-09-15T03:51:51.553739+00:00

## 自由属性

- 無し

