# n-ff2b scope rename (a): 形式の版 2 と写し、ログの変更の種類 scope-renamed と再生

- 種類: need
- scope: gy05
- created: 2026-09-15
- 状態: closed

## 関係

- spawned-by d-6e70 スコープ名の変更を操作 scope rename として閉じた集合に足す。履歴には「スコープ名の変更 旧 → 新（n ノード）」の 1 件の変更として残し、gy が gy.toml の [scopes.旧] を [scopes.新] に書き換える
- targets ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける
- targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
- targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている

## 本文

## 閉じ方

- 事実で閉じた（2026-09-15 lead が検収。measure 違反 0（score 366）、テスト 210 件 0 failed。gy 自身の正本の写し（版 1）を新しいバイナリで開くと format.1.bak と events.jsonl.1.bak を残して版 2 になり、handover と next が同じ値。コミット 9f04bbc）

## 自由属性

- 無し

