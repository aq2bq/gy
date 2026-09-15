# n-3f44 (N-44) render と HTML 投影（html.rs、ui/、e2e/、テスト、同梱物、CI）を最初に消し、しがらみの無い木で始める

state: closed
scope: gy05
created: 2026-09-15
  spawned-by d-9764 (D-81) N-44 の契約: render サブコマンドと HTML 投影を丸ごと消し、gy.toml の [render] は N-35 まで読み飛ばし、差分の上限は追加・変更行だけに掛ける
  targets ac-d2ab (AC-57) render サブコマンド、html.rs、ui/、e2e/、html_render のテスト、同梱の dist と template、CI の該当ジョブが無く、cargo test --workspace と cargo package --list にそれらが現れない
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている


閉じた理由: 事実（migrated: complete）

