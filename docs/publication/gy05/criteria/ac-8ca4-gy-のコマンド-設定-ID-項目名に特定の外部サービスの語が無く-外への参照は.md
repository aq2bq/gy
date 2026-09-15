# ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ

satisfied: true (2026-09-15 lead が grep。crates/gy-ledger/src と crates/gy5/src に github / issue / pull の語は 1 か所（../../crates/gy-ledger/src/ops/config.rs:8、コメント）で、コマンド・設定・ID・項目名には無い。gy5 --help に 0 件。外への参照は要求の ref 1 つ（D-73）) at 2026-09-15T03:51:51.290018+00:00
scope: gy05
created: 2026-09-14
  targeted-by n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）
  targeted-by n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割）
  targeted-by n-66ee (N-34) 要求の状態を 4 つにし、確定後は改訂・完了・中止の記録だけを持ち、ID を振り直して ref を持たせる
  targeted-by n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正



