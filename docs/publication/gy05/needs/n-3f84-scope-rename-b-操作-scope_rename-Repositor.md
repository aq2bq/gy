# n-3f84 scope rename (b): 操作 scope_rename（Repository 経由の適用、履歴 1 行、undo、Outcome）とテスト（n-ff2b から分割）

- 種類: need
- scope: gy05
- created: 2026-09-15
- 状態: closed

## 関係

- spawned-by d-6e70 スコープ名の変更を操作 scope rename として閉じた集合に足す。履歴には「スコープ名の変更 旧 → 新（n ノード）」の 1 件の変更として残し、gy が gy.toml の [scopes.旧] を [scopes.新] に書き換える
- targets ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける
- targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す

## 本文

## 閉じ方

- 事実で閉じた（2026-09-15 lead が検収。measure 違反 0（score 205）、テスト 0 failed。コミット a3fe323）

## 自由属性

- 無し

