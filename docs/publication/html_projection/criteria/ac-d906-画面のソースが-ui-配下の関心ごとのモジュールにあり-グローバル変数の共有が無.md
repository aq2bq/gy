# ac-d906 (AC-30) 画面のソースが ui/ 配下の関心ごとのモジュールにあり、グローバル変数の共有が無く、bun build の成果物がコミット済みで CI が再現一致を検査し、再構成の前後で e2e の全テストが通る

- 種類: criterion
- scope: html_projection
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-30

## 関係

- targeted-by n-b4d3 (N-25) 画面のコードを ui/ 配下の TypeScript モジュールに再構成し、bun で束ねた成果物を html.rs が埋め込む形にする

## 本文

## 充足

- satisfied（{"bun": "1.4.0", "bundle_check_clean_exit": 0, "bundle_check_injected_source_mismatch_exit": 1, "bundle_check_restored_exit": 0, "cargo_build_without_bun_path_exit": 0, "cargo_package_files": ["src/html/dist/app.js", "src/html/dist/app.css", "ui/src/templates/head.html", "ui/src/templates/body.html", "ui/src/templates/tail.html"], "ci": {"act_execution": "not run: no Docker connection", "act_workflow_parse_exit": 0, "github_actions": "not run: push is not authorized", "same_command_local_exit": 0, "workflow": "html-e2e / Check committed HTML bundle"}, "clippy_exit": 0, "commit": "f47b0c8656d946faea191cac6cfaa825874eeefb", "dependencies": 0, "dist_bytes": {"app.css": 18951, "app.js": 72525}, "docs": [{"diff": "ordered closure fragments -> ES modules, state ownership, pinned Bun, committed dist, CI check", "file": "docs/architecture.md", "section": "HTML source assembly"}, {"diff": "new contributor instructions, Cargo independence, modules and VM tests", "file": "crates/gy-core/ui/README.md", "section": "Build/check and Source responsibilities"}], "e2e_files_changed": 0, "e2e_passed": 28, "e2e_seconds": 45.0, "fmt_exit": 0, "global_or_imported_state_writes_outside_state": 0, "migration": "none for users; UI contributors need Bun 1.4.0 and commit regenerated dist", "modules_lines": {"components.ts": 3, "data.ts": 32, "detail/index.ts": 141, "detail/markdown.ts": 224, "dom.ts": 27, "filters.ts": 110, "graph/clusters.ts": 25, "graph/draw.ts": 376, "graph/layout.ts": 133, "graph/selection.ts": 83, "graph/svg.ts": 56, "graph/viewport.ts": 134, "list.ts": 36, "location.ts": 52, "main.ts": 24, "navigation.ts": 86, "state.ts": 151, "style.css": 173, "survey/blockers.ts": 52, "survey/index.ts": 4, "survey/overview.ts": 40, "survey/progress.ts": 65, "tokens.css": 80, "tokens.ts": 5}, "node_module_tests_passed": 10, "ownership_method": "TypeScript AST: imports from state; assignment/delete/mutating calls against those imports or window/globalThis; 22 TS modules; /private/tmp/gy-n25-ownership.cjs", "publish": "not run", "push": "not run", "rust_passed": 71, "acceptance_basis": "D-46: CIと同一のbun run checkをローカル実行。一致exit 0、生成元CSSの不一致注入exit 1、復元exit 0。GitHub実行は未実施。", "followup": "push後に開発担当がGitHubのhtml-e2e実行結果を確認し、run URL・対象commit・結果をAC-30.followup_evidenceへ追記する。"}） 2026-09-13T14:16:19.962640+00:00

## 自由属性

- 無し

