# d-9764 (D-81) N-44 の契約: render サブコマンドと HTML 投影を丸ごと消し、gy.toml の [render] は N-35 まで読み飛ばし、差分の上限は追加・変更行だけに掛ける

- 種類: decision
- scope: gy05
- created: 2026-09-15
- 別名: D-81

## 関係

- spawns n-3f44 (N-44) render と HTML 投影（html.rs、ui/、e2e/、テスト、同梱物、CI）を最初に消し、しがらみの無い木で始める

## 成立範囲

N-44 の依頼書（.local/brief-n44.md）の契約。2026-09-15 lead の判断。(1) 消す範囲は AC-57 のとおり: render サブコマンド（markdown / dot / html の 3 形式とも）、MCP の gy_render、crates/gy-core/src/html.rs と src/html/dist、crates/gy-core/ui/、e2e/、crates/gy/tests/html_render.rs、他テストの render を使う箇所、CI の html-e2e ジョブと bun の手順、docs/architecture.md の HTML projection 節、README 日英・CHEATSHEET・e2e/README.md の render と HTML の記述。(2) gy.toml の [render] テーブルと RenderConfig の読み込みは残し、init が [render] と .gitignore への html_output の追記を書くのをやめる。読み込みを残す理由は、既存の台帳（gy 自身と Kokopelli）が今のまま開けること。キーの拒否は N-35 で行う。(3) D-76 の差分 600 行は、このニーズでは追加行と既存ファイルの変更行に掛け、ファイル丸ごとの削除行は数えない。理由は、丸ごとの削除に読まずに書く危険が無いこと。(4) 消したことは CHANGELOG の Unreleased に書き、版は 0.5.0 の非互換の一つにまとめる（roadmap 1-7）。(5) AGENTS.md の e2e/README.md と terminal-browser への参照は lead が消す。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

