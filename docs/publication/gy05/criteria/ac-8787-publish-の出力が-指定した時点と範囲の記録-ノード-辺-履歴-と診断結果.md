# ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる

satisfied: true (2026-09-15 lead。gy -C docs/ledger publish --out docs/publication/{seq}.md で docs/publication/17.md（4,586 行）を生成しコミット fb3ebf1。原本を見ずに読み通せる完結した文書: 見出し（生成日時、seq 17、範囲、書き手、正本の場所）、読み方（5 種、4 状態、12 の関係、閉じ方、ID と別名、履歴の項目）、記録（258 ノードを scope・種類・created・ID 順で逐語、辺は両向きで相手の題名付き）、履歴（seq 1〜17 の書き込み単位を表で、ノードは ID + 題名）、診断（errors 0、warnings 1 件に解き方）。ID 単独の行 0 件（テストで保証）。マスターの指摘（振り返りの記録、原本を読まずに理解できる）に沿う) at 2026-09-15
scope: gy05
created: 2026-09-14
  targeted-by n-02b9 WebUI（第 3 段）: マスターが「これは何で何と関係があるか」「今マスター待ちは何か」を把握できる画面。問いは D-85 の 6 つを入力にし、作るのはマスターの指示があってから
  targeted-by n-2f21 (N-40) publish (a): views/publish の骨格と「いま判断待ち」（問い 2）、「未決の論点」（問い 6）、「注意」の節
  targeted-by n-45ef publish (a): スコープごとのディレクトリ出力と 1 ノード 1 ファイル（関係と mark を相手の題名付きで。対象スコープのディレクトリだけを作り直す）
  targeted-by n-5091 (N-71) publish (b): ノードの節（問い 1・3・4）と「この期間の変更」（問い 5）、gy5 の配線、publish-sample.md（N-40 から分割）
  targeted-by n-5c59 publish (b): スコープごとの索引 README.md（見出し・読み方・一覧とリンク・履歴・診断）と --since、gy.toml の output、文書、gy 自身の docs/publication の作り直し（n-45ef から分割）
  targeted-by n-6eae publish <ID> を見出しと箇条書きの形にし、既定の読み物の見本を 2 つ（判断待ちが無いとき / あるとき）作ってマスターに見せる
  targeted-by n-8f3f publish を記録の公開物にする: 指定した時点と範囲のノード・辺・履歴と診断結果を 1 ファイルに出し、--out（gy.toml の output）へ書いてコミットできる形にする。1 ページ化（N-72）は戻す
  targeted-by n-b6f3 (N-72) publish を 1 ページの読み物に直す: 既定はいま判断待ち・未決・注意の件数・期間の変更だけ、ノードの記述は publish <ID>... で指定した分だけ



