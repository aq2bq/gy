# ac-7145 (AC-37) ワークフローの外で閉じた要求を、実在しない作業記録を作らずに証跡付きで complete へ進められ、ワークフローを通る要求の complete のガードは変わらず、完了後の lint / handover が履歴を再検査しても指摘を出さない

satisfied: true (2026-09-14 写しの台帳で測定（target/debug/gy、D-57 の実装）。同梱の workflow.toml（complete を含むガード 5 つ）をそのまま入れた台帳で、記録の無い要求 #5969 の complete への advance は approval / audit_records / design_proposal の不足で終了コード 2。全ガードに waived_by = legacy_closure を書いても記録が無ければ同じく止まる。有効な legacy_closure（reason / evidence / closed_on / approver）を書くと advance が通り、transitions[].workflow.waived に 5 ガード全部が legacy_closure で残り、checks は 0 件、records は deviations と legacy_closure。その後 gy lint は無指摘で終了コード 0、handover の missing は空で lint 0 件。同じ profile の別要求 #6001 は記録が無ければ従来どおり止まり、approver が空の壊れた免除記録でも止まる。cargo fmt --check OK、clippy -D warnings OK、cargo test --workspace --locked 75 passed / 0 failed（configured_workflow 17、html_render 7、mutation_output 3、workflows 39、gy_core 9）。) at 2026-09-14T03:21:53.045954+00:00
scope: guard_waiver
created: 2026-09-14
  targeted-by n-7681 (N-32) ワークフローの外で進んだ要求 (legacy) を、証跡付きで complete へ進められる経路を用意する



