# ac-2101 (AC-21) 変更系コマンドの JSON 出力に入力として渡した本文が含まれず、CLI と MCP で同じ形を返す

- 種類: criterion
- scope: agent_footprint
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-21

## 関係

- targeted-by n-adaa (N-17) 変更系コマンドの返却を識別子と更新項目に絞り、本文は show で取得する形にする

## 本文

## 充足

- satisfied（N-17 / D-34 / D-36. Mutation summaries implemented for criterion add/satisfy, need add/file, question add/close, decide, link, req add/advance/compress with evidence, node set/submit, scope rename, init, import, plus q and gate add (also previously returned full nodes). No-evidence compression preview and query output unchanged. Added tests in crates/gy/tests/mutation_output.rs: body_file_results_match_cli_and_mcp_without_echoing_input; creation_satisfaction_and_links_return_only_changed_attribute_names; decision_and_question_mutations_preserve_warning_results. Tests cover body-only edits, an attribute named body, no-op updates, exact CLI/MCP result equality, compact human output, created/updated attribute names, evidence omission, warning preservation; existing compression tests also verify removed attribute names and absent archive. Before: 68 passed, 0 failed (5cb1e0a, /private/tmp/gy-n16-after-tests.log). After: 71 passed, 0 failed, 0 ignored (/private/tmp/gy-n17-tests.log). cargo fmt --all -- --check exit 0; cargo clippy --workspace --all-targets --locked -- -D warnings exit 0. Measured 34,000-byte input body => 101-byte mutation JSON, with full body retained by show (/private/tmp/gy-n17-output-measurement.json). Snapshot comparison retains at most two returned nodes per ordinary mutation; scope rename scans V nodes once and retains only nodes in the renamed scope, then compares each returned node once (attributes and their values/body, including history if present); query operations retain none. CHANGELOG.md Unreleased/Breaking and README.md / README.ja.md command-output paragraphs match the implemented result keys and show migration; bundled skills contain no old response-shape dependencies.） 2026-09-13T12:19:33.181655+00:00

## 自由属性

- 無し

