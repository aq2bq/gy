# n-adaa (N-17) 変更系コマンドの返却を識別子と更新項目に絞り、本文は show で取得する形にする

- 種類: need
- scope: agent_footprint
- created: 2026-09-13
- 状態: closed
- 別名: N-17

## 関係

- spawned-by d-c956 (D-34) 変更系コマンドは id と更新した属性名だけを返し、本文と全属性は show / find が返す
- targets ac-2101 (AC-21) 変更系コマンドの JSON 出力に入力として渡した本文が含まれず、CLI と MCP で同じ形を返す

## 本文

## 出所

N-16 と同じ報告。デック先生は N-11 の設計案を `gy node set Q-25 --body-file` で保存し、コマンドが返す JSON に本文全体が含まれていたため、直前に生成した長文が再びモデルへ戻った。

## 観測

`crates/gy/src/main.rs` の `NodeCommand::Set` は更新後に `store.node(id)` 全体を `{"node": ...}` で返す。`--quiet` は通常出力を抑えるが JSON 結果には本文が残る。MCP も同じ操作を利用する。

## 判定

ツールの問題としてツールで解く。返却の範囲は出力契約の変更を含むため Q-27 でマスターの判断を待つ。

## 要求（2026-09-13、マスター承認済み。開発担当 deck へ振り出し）

根拠は上の観測と D-34。状態は `main` 5cb1e0a（0.4.0 公開後、未公開の互換変更 2 コミットを含む）。判定は AC-21 の evidence で行う。

### 契約

- 変更系コマンドの結果は「何が変わったか」を返す。識別子（`id`、`type`、`scope`）と、変更した属性名の一覧（作成時は作成された属性名）。本文と属性の値は返さず、`show` / `find` が返す。
- 入力として渡したもの（`--body-file` の本文、`--set` の値、`--evidence` の文、submit の記録内容）は結果に含めない。結果に残るのは、操作が新たに生んだ情報だけである: ID、変更した属性名、警告（`search_hits`、`open_questions`、`warnings`）、submit の revision と記録名。
- 対象は `crates/gy/src/main.rs` で `output.value` にノード全体を入れている全ての変更系: criterion add / satisfy、need add / file、question add / close、decide、link、req add / advance / compress、node set / submit、scope rename、init、import。参照系（show、find、handover、next、lint、render、stats、cheatsheet）は無変更。
- MCP は同じ `Output` を `structuredContent` に包むので、CLI `--json` と MCP `tools/call` の結果は同じ形になる。人間向け出力（`--json` なし）も同じ情報を1行〜数行で出す。

### 維持する条件

- 保存形式、diagnostics と終了コード、`gy.toml` の解釈、参照系の出力は無変更。
- 変更した契約はテストで守る: `node set --body-file` の結果に本文が無いこと、同じ操作の CLI `--json` と MCP の結果が一致すること、警告が引き続き返ること。既存テストで `["node"]` を読んでいる箇所の期待値変更は D-34 を理由として行う。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked` が exit 0。

### 文書

- `CHANGELOG.md` に `## Unreleased` を置き、Breaking として出力契約の変更と移行（全属性と本文は `gy show <ID> --json`）を書く。バージョン番号は変えない（リリース時に整合する）。
- `README.md` と `README.ja.md` の出力の説明を両方とも同期する。同梱スキル（`crates/gy/skills/`）に結果の形へ依存する記述があれば同期する。

### 完了

- AC-21 を `gy criterion satisfy --evidence` で閉じる。evidence は、変更した変更系コマンドの一覧、追加したテスト名、前後のテスト件数、fmt / clippy の終了コード。
- 変更はコミットまで進め、push と公開はマスターの指示を待つ。コミットメッセージは英語で変更理由を書く。
- 報告は `herdr agent prompt kuroko` で、D-33 の形で送る。判断に迷う点は question として台帳に置いて私へ送る。

## 完了確認（kuroko、2026-09-13）

deck の報告をコミット beb4449 で照合。kuroko 側の再実行: fmt exit 0、clippy 警告 0、cargo test --workspace --locked 71 passed / 0 failed（mutation_output.rs の 3 件を含む）。実機プローブ（一時台帳、本文 5,000 バイト）: node set --body-file の結果は {"node":{"body_changed":true,"changed_attributes":[],...}} の 1 行、同値再設定は body_changed=false、show の本文長は 5,000 で保持。CHANGELOG の Unreleased/Breaking、README.md 105 行付近と README.ja.md 103 行の出力説明が実装のキーと一致。AC-21 satisfied。push・公開は未実施。main は 0.4.0 から非互換変更を含むため次版は 0.5.0。

## 閉じ方

- 事実で閉じた（gy 0.5 の書きは Outcome（id・changed・missing・next）を返し本文は show（N-39）（d-736e））

## 自由属性

- 無し

