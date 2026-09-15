# n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割）

state: closed
scope: gy05
created: 2026-09-15
  depends-on n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正
  spawned-by d-be4c (D-70) 確定後の要求について gy が持つのは、承認後の改訂・完了・中止の 3 つの人による記録だけであり、gy は GitHub を読まない
  targets ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%）
  targets ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0
  targets ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  depended-on-by n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）


閉じた理由: 事実（migrated: complete）

