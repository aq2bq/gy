# n-35cf show / list / publish のニーズの状態を next と同じ導出（open / closed / done）にする。list --status done を受ける

state: closed
scope: gy05
created: 2026-09-15
  targets ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
閉じた理由: 事実（2026-09-15 lead が検収。measure 違反 0（score 100）、テスト 0 failed。Kokopelli の台帳（読むだけ）で show N-25 が state: done、list --type need --status done に N-24 と N-25。コミット 1714824）

