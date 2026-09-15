# d-6e70 スコープ名の変更を操作 scope rename として閉じた集合に足す。履歴には「スコープ名の変更 旧 → 新（n ノード）」の 1 件の変更として残し、gy が gy.toml の [scopes.旧] を [scopes.新] に書き換える

- 種類: decision
- scope: gy05
- created: 2026-09-15

## 関係

- spawns n-24dd scope rename (c): CLI と gy.toml の書き換え（コメント・順序を保ち、失敗時はログを戻す）、文書、gy 自身の写しでの確認（n-ff2b から分割）
- spawns n-3f84 scope rename (b): 操作 scope_rename（Repository 経由の適用、履歴 1 行、undo、Outcome）とテスト（n-ff2b から分割）
- spawns n-ff2b scope rename (a): 形式の版 2 と写し、ログの変更の種類 scope-renamed と再生
- widens d-1cfb (D-84) 新しい gy の操作の集合は末端 20（書き 15、読み 5）、固有オプション 30 以下とし、AC-49 をこの数に改める（mark: undo は AC-46 が求める一つの意図で独立したコマンドにする）

## 成立範囲

2026-09-15 マスターの決定（B、gy が書く）。理由: スコープを気軽に変えられないと使い勝手が一段悪い。履歴が実際に読まれる場面（list --actor / --since、公開物の履歴、undo）はどれも期間や書き手で切る読み方で、ノードごとの変遷を辿る画面は無い。全ノードを更新する形（A）だと list --since がノード数の行で埋まる。B はログの変更の種類が 1 つ増えるので形式の版を上げ、古いログの読み込みを同梱する（D-77）。gy.toml は gy が書き換える（人が先に直すと gy.toml とノードがずれる中途半端な状態が起きるため）。末端は 21（書き 16）になり AC-49 を改める（D-84 を widens）。選ばなかった案: A 全ノード更新、C 別名で吸収（正本と表示がずれる）。

## 本文

## 自由属性

- 無し

