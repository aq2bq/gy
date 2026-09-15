# ac-d2ab (AC-57) render サブコマンド、html.rs、ui/、e2e/、html_render のテスト、同梱の dist と template、CI の該当ジョブが無く、cargo test --workspace と cargo package --list にそれらが現れない

satisfied: true (2026-09-15 lead が検収。判定 1: target/debug/gy render の終了コード 2、cargo package --list -p gy-core に html / ui / dist / templates 0 件、crates/gy-core/ui・e2e・src/html.rs・src/html/ が無い、ci.yml に html-e2e / playwright / bun の手順無し、cargo test の出力に html_render 0 件、render の残りは model.rs と store.rs の [render] 読み込み・検証・書き出し時の除去だけ。判定 2: docs/ledger で lint と handover が no findings。判定 3: 残るファイルへの追加 48 行 + 新テスト 35 行（基準 600。残るファイル内の削除 568 行と丸ごとの削除は数えない）、新テスト 35 行（300）、glob import 10 → 9、触ったファイルは依頼書の許可内 + 許可した graph.rs の 1 行。fmt / clippy / test 通過、テスト 65 件 0 failed。消したテストは html_render.rs の 1 ファイルのみで、workflows.rs の 3 テストは render の部分だけ除いて復元。報告は .local/report-n44.md) at 2026-09-15T00:24:55.604198+00:00
scope: gy05
created: 2026-09-15
  targeted-by n-3f44 (N-44) render と HTML 投影（html.rs、ui/、e2e/、テスト、同梱物、CI）を最初に消し、しがらみの無い木で始める



