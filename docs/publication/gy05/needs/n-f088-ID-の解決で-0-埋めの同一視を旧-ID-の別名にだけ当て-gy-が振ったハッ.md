# n-f088 ID の解決で、0 埋めの同一視を旧 ID の別名にだけ当て、gy が振ったハッシュ ID には当てない（d-0008 が D-8 と衝突する欠陥）。新しい ID は数字だけのハッシュを避ける

state: closed
scope: gy05
created: 2026-09-15
  targets ac-6251 (AC-50) 利用者の文書と運用に採番の規則が現れない。ID の衝突と振り直しが起きない
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
閉じた理由: 事実（2026-09-15 lead が検収。measure 違反 0（score 83）、テスト 0 failed。Kokopelli の台帳（読むだけ）で show D-8 が d-43e5、show d-0008 が d-0008、show D-08 が d-43e5。コミット 96dcfd3、ローカルにインストール）

