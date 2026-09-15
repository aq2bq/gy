# gy の公開物 — html_projection

- 生成: 2026-09-15T08:12:18Z
- seq: 51
- scope: html_projection
- since: (先頭から)
- 書き手: piko
- 正本: /Users/pememo/.local/share/gy/fae59914

## 読み方

ノードの種類（5 種）:
- need（n-…）: これから行う作業。受け入れ条件を対象にする。
- question（q-…）: まだ決めていないこと。決定者と 2 つ以上の選択肢を持つ。
- decision（d-…）: 条件付きの決定。成立範囲（decision_scope）を持つ。
- requirement（r-…）: ニーズを承認に回したもの。外への参照（ref）を持つ。
- criterion（ac-…）: 受け入れ条件。作業の進みを数える相手。

要求の状態（4 つ）:
- filed: 起票済み。設計はまだ承認されていない。
- approved: 確定。マスターの設計承認がある。
- done: 完了。出荷の根拠が記録されている。
- cancelled: 中止。完了以外の理由で閉じた。

関係（12）と向き:
- closes: 論点 → 決定。
- narrows / widens / supersedes / completes: 決定 → 決定。
- targets: ニーズ / 要求 → 受け入れ条件。
- spawned-by: ニーズ → 決定。
- filed-as: ニーズ → 要求。
- depends-on: ニーズ → ニーズ。
- relies-on: 要求 → 決定。
- raised: 要求 → 論点。
- waits-on: ニーズ → 論点・要求。
辺は始点のノードだけに保存し、逆向きは導出する。この文書は両向きを出す。

論点の閉じ方:
- fact: 事実で閉じた。 - decision: 決定で閉じた。 - non-decision: 決定を伴わずに閉じた。

ID と別名:
- ID は種類の接頭辞と短いハッシュ（例 n-3f9a）。旧 ID があれば別名として併記する（例 D-78）。
- 参照はすべて「ID（別名） 題名」で書く。要求は ref（外への参照）も持つ。

履歴の項目:
- seq: 書き込みの連番（時点）。 - 日時・書き手: いつ・誰が（GY_ACTOR）。
- ノード: 対象の ID と題名。 - 何を: created / updated / deleted。
- なぜ: 操作の名前と対象。 - 出典: 操作に渡した根拠や URL。

## 一覧

### needs
- [n-0228 (N-6) 現在の焦点までの経路を画面に出し、任意の段へ戻せるようにする](needs/n-0228-現在の焦点までの経路を画面に出し-任意の段へ戻せるようにする.md) — closed
- [n-1b72 (N-27) 押せる見た目と押せる実体を一致させ、Overview の数値カードを一覧への入口にする](needs/n-1b72-押せる見た目と押せる実体を一致させ-Overview-の数値カードを一覧への入口.md) — closed
- [n-2e91 (N-8) 詳細パネルをグラフ領域と同等の幅にする](needs/n-2e91-詳細パネルをグラフ領域と同等の幅にする.md) — closed
- [n-3f2d (N-26) ツールバーと状態表示を設計トークンに基づいて組み直し、絞り込み・グラフ操作・リセットを分ける](needs/n-3f2d-ツールバーと状態表示を設計トークンに基づいて組み直し-絞り込み-グラフ操作-リセ.md) — closed
- [n-4759 (N-24) タイトルを一覧と詳細で全文、グラフのラベルで折り返し表示にする](needs/n-4759-タイトルを一覧と詳細で全文-グラフのラベルで折り返し表示にする.md) — closed
- [n-5ba8 (N-28) 履歴の戻る/進む直後に hash と表示状態が一致することを、時間依存なく成立させる](needs/n-5ba8-履歴の戻る-進む直後に-hash-と表示状態が一致することを-時間依存なく成立さ.md) — closed
- [n-7170 (N-3) バナーを1つに統合し、俯瞰では clusters、個別では nodes と書き分ける](needs/n-7170-バナーを1つに統合し-俯瞰では-clusters-個別では-nodes-と書き分.md) — closed
- [n-76b4 (N-1) 集約辺のラベルを、対応する曲線の近くへ置く](needs/n-76b4-集約辺のラベルを-対応する曲線の近くへ置く.md) — closed
- [n-8619 (N-7) playwright 相当のE2Eを開発専用として導入し、CI から実行できるようにする](needs/n-8619-playwright-相当のE2Eを開発専用として導入し-CI-から実行できるよ.md) — closed
- [n-8c5e (N-2) フィットの計算にラベルの広がりを含め、端でラベルが切れないようにする](needs/n-8c5e-フィットの計算にラベルの広がりを含め-端でラベルが切れないようにする.md) — closed
- [n-ac8e (N-21) 検索と絞り込みをヘッダー直下の常設ツールバーにし、左パネルを状況把握に限る](needs/n-ac8e-検索と絞り込みをヘッダー直下の常設ツールバーにし-左パネルを状況把握に限る.md) — closed
- [n-afb6 (N-5) 焦点を履歴のあるスタックにし、ノードのクリックを焦点の移動とする](needs/n-afb6-焦点を履歴のあるスタックにし-ノードのクリックを焦点の移動とする.md) — closed
- [n-afe9 (N-19) 表示状態を URL の hash に載せ、ID で直接到達でき、戻るが効くようにする](needs/n-afe9-表示状態を-URL-の-hash-に載せ-ID-で直接到達でき-戻るが効くように.md) — closed
- [n-b4d3 (N-25) 画面のコードを ui/ 配下の TypeScript モジュールに再構成し、bun で束ねた成果物を html.rs が埋め込む形にする](needs/n-b4d3-画面のコードを-ui-配下の-TypeScript-モジュールに再構成し-bun.md) — closed
- [n-c3b4 (N-10) 成立範囲と本文を読める行長の読み物として配置する](needs/n-c3b4-成立範囲と本文を読める行長の読み物として配置する.md) — closed
- [n-ca04 (N-22) 絞り込み結果の一覧ビューを置き、クラスタと種別のクリックを一覧へ導く](needs/n-ca04-絞り込み結果の一覧ビューを置き-クラスタと種別のクリックを一覧へ導く.md) — closed
- [n-cb5e (N-29) 辺ラベルがノードのタイトルと重ならないようにし、重なる場合は選択した辺だけに出す](needs/n-cb5e-辺ラベルがノードのタイトルと重ならないようにし-重なる場合は選択した辺だけに出す.md) — closed
- [n-cda9 (N-23) 詳細本文の Markdown 描画を GFM の表・入れ子リスト・コード・リンクに対応させる](needs/n-cda9-詳細本文の-Markdown-描画を-GFM-の表-入れ子リスト-コード-リンク.md) — closed
- [n-f025 (N-20) ノードのクリックを詳細表示だけにし、焦点移動と絞り込みを明示操作にする](needs/n-f025-ノードのクリックを詳細表示だけにし-焦点移動と絞り込みを明示操作にする.md) — closed
- [n-f20e (N-9) メタ情報をバッジとラベルで構造化する](needs/n-f20e-メタ情報をバッジとラベルで構造化する.md) — closed
### questions
- [q-041d (Q-14) template.html の分割を、ナビゲーションの修正より前に行うか後に行うか](questions/q-041d-template-html-の分割を-ナビゲーションの修正より前に行うか後に行う.md) — closed
- [q-103c (Q-4) non_exhaustive を RenderConfig だけに付けるか、公開設定型すべてに付けるか](questions/q-103c-non_exhaustive-を-RenderConfig-だけに付けるか-公開.md) — closed
- [q-16f6 (Q-22) CI の対象プラットフォームと性能の関門をどう置くか](questions/q-16f6-CI-の対象プラットフォームと性能の関門をどう置くか.md) — closed
- [q-1aa0 (Q-21) 詳細の等幅配置でAC-12の行長下限をどの画面幅と文字幅に適用するか](questions/q-1aa0-詳細の等幅配置でAC-12の行長下限をどの画面幅と文字幅に適用するか.md) — closed
- [q-1dbb (Q-16) 描画領域の大きさだけが変わったときに再フィットするか](questions/q-1dbb-描画領域の大きさだけが変わったときに再フィットするか.md) — closed
- [q-1eb0 (Q-8) グラフの描き分けはズーム段階で切り替えるか、表示対象の件数で切り替えるか](questions/q-1eb0-グラフの描き分けはズーム段階で切り替えるか-表示対象の件数で切り替えるか.md) — closed
- [q-231e (Q-13) グラフの浮上と降下を、モードの切り替えで作るかモードレスな焦点の移動で作るか](questions/q-231e-グラフの浮上と降下を-モードの切り替えで作るかモードレスな焦点の移動で作るか.md) — closed
- [q-281b (Q-17) 焦点の1ホップが60件を超えるときクラスタへ戻すか個別表示を継続するか](questions/q-281b-焦点の1ホップが60件を超えるときクラスタへ戻すか個別表示を継続するか.md) — closed
- [q-32af (Q-2) 生成したHTMLをバージョン管理へ入れるか](questions/q-32af-生成したHTMLをバージョン管理へ入れるか.md) — closed
- [q-3d01 (Q-9) 近傍展開が使う辺集合は、画面が描く EDGES か、frontmatter の全参照か](questions/q-3d01-近傍展開が使う辺集合は-画面が描く-EDGES-か-frontmatter-の全.md) — closed
- [q-4473 (Q-32) 常設表の行リンク検査は実際のヒット対象で行うかtrをlinkへ変更するか](questions/q-4473-常設表の行リンク検査は実際のヒット対象で行うかtrをlinkへ変更するか.md) — closed
- [q-4b50 (Q-19) 要求にしないまま終わったニーズを next から外す手段が無い](questions/q-4b50-要求にしないまま終わったニーズを-next-から外す手段が無い.md) — closed
- [q-4dd9 (Q-11) init が既存の gy.toml を正規化して書き直す挙動を変えるか](questions/q-4dd9-init-が既存の-gy-toml-を正規化して書き直す挙動を変えるか.md) — closed
- [q-54ea (Q-20) 詳細パネルを読み物として成立させるか、属性の一覧として保つか](questions/q-54ea-詳細パネルを読み物として成立させるか-属性の一覧として保つか.md) — closed
- [q-5921 (Q-33) N-24の全ラベル画面内収容はLineageの世代配置にも適用するか](questions/q-5921-N-24の全ラベル画面内収容はLineageの世代配置にも適用するか.md) — closed
- [q-6838 (Q-30) N-25 の CI 再現検査はローカル実行の結果で受け入れるか、GitHub 実行まで待つか](questions/q-6838-N-25-の-CI-再現検査はローカル実行の結果で受け入れるか-GitHub-実.md) — closed
- [q-6d40 (Q-31) Overviewから選ぶnext集合をURLで意味として保存するかID集合として保存するか](questions/q-6d40-Overviewから選ぶnext集合をURLで意味として保存するかID集合として.md) — closed
- [q-6fbc (Q-3) ノード本文をHTMLへ全文埋め込むか、要約して遅延読込にするか](questions/q-6fbc-ノード本文をHTMLへ全文埋め込むか-要約して遅延読込にするか.md) — closed
- [q-7234 (Q-6) render の終了コードは lint の結果を反映すべきか](questions/q-7234-render-の終了コードは-lint-の結果を反映すべきか.md) — closed
- [q-735e (Q-15) HTMLの振る舞いの検証に何を使うか](questions/q-735e-HTMLの振る舞いの検証に何を使うか.md) — closed
- [q-83bf (Q-7) 0.3.0 をドッグフーディングのフィードバックより前に公開するか](questions/q-83bf-0-3-0-をドッグフーディングのフィードバックより前に公開するか.md) — closed
- [q-83d4 (Q-12) gy 自身の台帳をバージョン管理へ入れるか](questions/q-83d4-gy-自身の台帳をバージョン管理へ入れるか.md) — closed
- [q-8aca (Q-1) 台帳をHTMLへ投影する入口は render の追加オプションか、新しいコマンドか](questions/q-8aca-台帳をHTMLへ投影する入口は-render-の追加オプションか-新しいコマンド.md) — closed
- [q-a09e (Q-5) Default を持たない RecordSchema / FieldSchema / RecordCheck に Default を足すか](questions/q-a09e-Default-を持たない-RecordSchema-FieldSchema-R.md) — closed
- [q-ab52 (Q-10) 近傍展開のホップ数を固定するか、起点ごとに可変にするか](questions/q-ab52-近傍展開のホップ数を固定するか-起点ごとに可変にするか.md) — closed
- [q-e833 (Q-34) N-29 の選択辺フォールバックは新設するか、既存のノード詳細へ委ねるか](questions/q-e833-N-29-の選択辺フォールバックは新設するか-既存のノード詳細へ委ねるか.md) — closed
- [q-fd63 (Q-18) E2E と動作保証の対象ブラウザをいくつにするか](questions/q-fd63-E2E-と動作保証の対象ブラウザをいくつにするか.md) — closed
### decisions
- [d-0325 (D-50) 辺ラベルはノードのタイトルと衝突するとき隠し、関係はノード詳細の関係一覧で読む。辺の選択という操作状態は設けない](decisions/d-0325-辺ラベルはノードのタイトルと衝突するとき隠し-関係はノード詳細の関係一覧で読む.md)
- [d-0621 (D-25) HTMLの自己完結性は読込時と描画操作時に外部取得を行わないことで検証する](decisions/d-0621-HTMLの自己完結性は読込時と描画操作時に外部取得を行わないことで検証する.md)
- [d-1a18 (D-40) 一覧が主、グラフは関係を見る補助である](decisions/d-1a18-一覧が主-グラフは関係を見る補助である.md)
- [d-2455 (D-2) 生成したHTMLはローカル生成のみとし、init が .gitignore へ追記する](decisions/d-2455-生成したHTMLはローカル生成のみとし-init-が-gitignore-へ追記.md)
- [d-29e7 (D-17) 再フィットは現在の変換の由来で分け、1回の操作につき1回に限る](decisions/d-29e7-再フィットは現在の変換の由来で分け-1回の操作につき1回に限る.md)
- [d-2c8a (D-15) template.html の分割をナビゲーションの修正より前に行う](decisions/d-2c8a-template-html-の分割をナビゲーションの修正より前に行う.md)
- [d-3026 (D-41) 検索と絞り込みは常設のツールバーとしてヘッダー直下に置く](decisions/d-3026-検索と絞り込みは常設のツールバーとしてヘッダー直下に置く.md)
- [d-36b2 (D-6) lint の結果は render の終了コードを変えない](decisions/d-36b2-lint-の結果は-render-の終了コードを変えない.md)
- [d-3755 (D-1) HTML投影の入口は render --format html とする](decisions/d-3755-HTML投影の入口は-render---format-html-とする.md)
- [d-3c53 (D-14) 浮上と降下は、焦点と半径だけを持つモードレスな構成で実現する](decisions/d-3c53-浮上と降下は-焦点と半径だけを持つモードレスな構成で実現する.md)
- [d-42c9 (D-22) E2E と動作保証の対象は Chromium だけとする](decisions/d-42c9-E2E-と動作保証の対象は-Chromium-だけとする.md)
- [d-4425 (D-39) モードレス: 同じ操作は状態に依らず同じ意味を持ち、クリックは選んで見る、配置や絞り込みの変更は明示操作である](decisions/d-4425-モードレス-同じ操作は状態に依らず同じ意味を持ち-クリックは選んで見る-配置や絞.md)
- [d-47e7 (D-42) 本文は台帳の Markdown を欠けずに描く](decisions/d-47e7-本文は台帳の-Markdown-を欠けずに描く.md)
- [d-51f6 (D-11) init が gy.toml を正規化して書き直す挙動は変えず、文書で伝える](decisions/d-51f6-init-が-gy-toml-を正規化して書き直す挙動は変えず-文書で伝える.md)
- [d-521b (D-10) 近傍展開のホップ数は起点ごとに閾値へ収まる最大値を選ぶ](decisions/d-521b-近傍展開のホップ数は起点ごとに閾値へ収まる最大値を選ぶ.md)
- [d-5766 (D-49) ラベルの全収容（文字高 11px 以上・交差 0・画面外 0）は通常の個別描画に適用し、Lineage は折り返しラベルと世代配置を保ち、可読倍率とパンで読む](decisions/d-5766-ラベルの全収容-文字高-11px-以上-交差-0-画面外-0-は通常の個別描画に.md)
- [d-5d63 (D-37) HTML projection が支える業務は、指名到達・絞り込んで一覧を読む・読む・辿る・状況把握の5つである](decisions/d-5d63-HTML-projection-が支える業務は-指名到達-絞り込んで一覧を読む.md)
- [d-626e (D-38) 場所は URL が持つ: 表示状態は hash に載り、ブラウザの戻るが効き、ID を貼れば開く](decisions/d-626e-場所は-URL-が持つ-表示状態は-hash-に載り-ブラウザの戻るが効き-ID.md)
- [d-6380 (D-48) 一覧の行は表の row 役割を保ち、行全体を覆う native link を実ヒット対象とする。押せる見た目の検査は要素の計算スタイルではなく実ヒット対象で行う](decisions/d-6380-一覧の行は表の-row-役割を保ち-行全体を覆う-native-link-を実ヒ.md)
- [d-6793 (D-18) 個別描画の自動フィットは倍率2を上限とし、手動ズームの上限4を維持する](decisions/d-6793-個別描画の自動フィットは倍率2を上限とし-手動ズームの上限4を維持する.md)
- [d-6acb (D-21) 焦点があるときは焦点を含む最大件数まで個別描画し、残りの件数と一覧への導線を示す](decisions/d-6acb-焦点があるときは焦点を含む最大件数まで個別描画し-残りの件数と一覧への導線を示す.md)
- [d-6e4d (D-44) 押せるものは押せる見た目を持ち、押せる見た目のものは押すと何かが起きる](decisions/d-6e4d-押せるものは押せる見た目を持ち-押せる見た目のものは押すと何かが起きる.md)
- [d-706f (D-47) hash は表示の指定を保持し、データは開いた HTML から導く。出所付きの一覧は種別化した出所（overview=next など）で表し、ID 集合は保存しない](decisions/d-706f-hash-は表示の指定を保持し-データは開いた-HTML-から導く-出所付きの一.md)
- [d-73a5 (D-19) 孤立した焦点にはグラフ上の接続が無いことを画面に示す](decisions/d-73a5-孤立した焦点にはグラフ上の接続が無いことを画面に示す.md)
- [d-84bc (D-3) ノード本文はHTMLへ全文埋め込む](decisions/d-84bc-ノード本文はHTMLへ全文埋め込む.md)
- [d-9270 (D-23) 詳細は宣言と説明を分けた読み物として構成し、グラフ領域と同等の幅を持たせる](decisions/d-9270-詳細は宣言と説明を分けた読み物として構成し-グラフ領域と同等の幅を持たせる.md)
- [d-9ad5 (D-12) ズーム段階に連動した3段階の詳細度でグラフを描き分ける](decisions/d-9ad5-ズーム段階に連動した3段階の詳細度でグラフを描き分ける.md)
- [d-a1a2 (D-26) CI は Linux のみで回し、自動操作の往復時間を関門にしない](decisions/d-a1a2-CI-は-Linux-のみで回し-自動操作の往復時間を関門にしない.md)
- [d-a2c6 (D-45) 画面のコードは関心ごとの ES モジュールと TypeScript で書き、bun で 1 本に束ねた成果物を html.rs が埋め込む](decisions/d-a2c6-画面のコードは関心ごとの-ES-モジュールと-TypeScript-で書き-bu.md)
- [d-a3e2 (D-9) 近傍展開の辺集合は画面が描く EDGES ベースとする](decisions/d-a3e2-近傍展開の辺集合は画面が描く-EDGES-ベースとする.md)
- [d-abe1 (D-20) 自動表示は可読性を優先し、焦点または表示対象の中心を基準に画面外件数を示す](decisions/d-abe1-自動表示は可読性を優先し-焦点または表示対象の中心を基準に画面外件数を示す.md)
- [d-affc (D-5) RecordSchema と FieldSchema と RecordCheck は読み取り専用型とし Default を実装しない](decisions/d-affc-RecordSchema-と-FieldSchema-と-RecordCheck.md)
- [d-b9ff (D-7) 0.3.0 はドッグフーディングを待たずに公開する](decisions/d-b9ff-0-3-0-はドッグフーディングを待たずに公開する.md)
- [d-ba72 (D-24) AC-12 の行長は文字体系ごとの慣用に合わせ、日本語は全角30から45文字、欧文は45から90文字とする](decisions/d-ba72-AC-12-の行長は文字体系ごとの慣用に合わせ-日本語は全角30から45文字-欧.md)
- [d-c567 (D-46) CI に追加した検査は、同一コマンドのローカル成功と欠陥注入での失敗を根拠に受け入れ、GitHub 上の実行結果は push 後に追補の evidence として確かめる](decisions/d-c567-CI-に追加した検査は-同一コマンドのローカル成功と欠陥注入での失敗を根拠に受け.md)
- [d-c6bd (D-43) タイトルは全文が読める場所を必ず持ち、グラフのラベルは折り返しを優先して省略を最後の手段とする](decisions/d-c6bd-タイトルは全文が読める場所を必ず持ち-グラフのラベルは折り返しを優先して省略を最.md)
- [d-ce87 (D-13) gy 自身の台帳はバージョン管理へ入れず、AGENTS.md に運用を定める](decisions/d-ce87-gy-自身の台帳はバージョン管理へ入れず-AGENTS-md-に運用を定める.md)
- [d-d764 (D-16) HTMLの振る舞いの検証に自動化されたE2Eを導入する](decisions/d-d764-HTMLの振る舞いの検証に自動化されたE2Eを導入する.md)
- [d-d97b (D-8) グラフの描き分けはズーム段階ではなく表示対象の件数で行う](decisions/d-d97b-グラフの描き分けはズーム段階ではなく表示対象の件数で行う.md)
- [d-f45a (D-4) non_exhaustive は公開設定型9種すべてに付ける](decisions/d-f45a-non_exhaustive-は公開設定型9種すべてに付ける.md)
- [d-4aa9 (D-80) html_projection の未着手ニーズ 14 件は D-79 により実装せずに閉じ、status を complete にして dropped_by で D-80 を指す](decisions/d-4aa9-html_projection-の未着手ニーズ-14-件は-D-79-により実装.md)
### requirements
- 無し
### criteria
- [ac-0b0b (AC-31) ツールバーが絞り込み・グラフ操作・リセットの3群に分かれて 1440px で 1 行に収まり、状態の表示（焦点・件数・絞り込み）が画面に 1 か所だけあり、凡例がグラフのノードを覆わない](criteria/ac-0b0b-ツールバーが絞り込み-グラフ操作-リセットの3群に分かれて-1440px-で-1.md) — satisfied
- [ac-1967 (AC-2) 俯瞰から個別描画へ至る経路が、ノードIDを知らない利用者の操作だけで成立する](criteria/ac-1967-俯瞰から個別描画へ至る経路が-ノードIDを知らない利用者の操作だけで成立する.md) — satisfied
- [ac-1da3 (AC-6) 分割の前後で、同じ台帳から生成されるHTMLが同一である](criteria/ac-1da3-分割の前後で-同じ台帳から生成されるHTMLが同一である.md) — satisfied
- [ac-1dcb (AC-5) 実台帳314ノードで、生成したHTMLがネットワークへ出ずに file:// で全機能動作する](criteria/ac-1dcb-実台帳314ノードで-生成したHTMLがネットワークへ出ずに-file-で全機能.md) — satisfied
- [ac-1fe0 (AC-7) 個別描画のノードをクリックすると焦点がそのノードへ移り、直前の焦点へ戻せる](criteria/ac-1fe0-個別描画のノードをクリックすると焦点がそのノードへ移り-直前の焦点へ戻せる.md) — satisfied
- [ac-3e7f (AC-25) ノードをクリックしてもグラフの配置と焦点は変わらず詳細だけが開き、焦点移動と絞り込みは押す前に結果が読める明示ボタンで行われる](criteria/ac-3e7f-ノードをクリックしてもグラフの配置と焦点は変わらず詳細だけが開き-焦点移動と絞り.md) — satisfied
- [ac-4392 (AC-9) 自動化されたE2Eが、クリックの到達と表示対象の変化に伴う再フィットを検出する](criteria/ac-4392-自動化されたE2Eが-クリックの到達と表示対象の変化に伴う再フィットを検出する.md) — satisfied
- [ac-4847 (AC-34) 個別描画で辺ラベルの矩形がノードのタイトル矩形と交差せず、交差する場合は辺ラベルを出さずに選択した辺だけで表示する](criteria/ac-4847-個別描画で辺ラベルの矩形がノードのタイトル矩形と交差せず-交差する場合は辺ラベル.md) — satisfied
- [ac-577d (AC-10) 詳細を開いたとき、詳細領域の幅がグラフ領域の幅の0.9倍から1.1倍の範囲に入る（広い画面）](criteria/ac-577d-詳細を開いたとき-詳細領域の幅がグラフ領域の幅の0-9倍から1-1倍の範囲に入る.md) — satisfied
- [ac-57a4 (AC-13) 未知属性が既知の宣言と分けた領域に残り、値が省略されない](criteria/ac-57a4-未知属性が既知の宣言と分けた領域に残り-値が省略されない.md) — satisfied
- [ac-5b53 (AC-24) gy.html#<ID> を開くとそのノードの詳細が開いており、焦点・選択・絞り込み・検索語の変更が hash に反映され、ブラウザの戻るで直前の表示状態に戻る](criteria/ac-5b53-gy-html-ID-を開くとそのノードの詳細が開いており-焦点-選択-絞り込み.md) — satisfied
- [ac-670c (AC-3) 表示対象が変わったときに自動でフィットし、その直後のノードラベルの文字高が11ピクセル以上である](criteria/ac-670c-表示対象が変わったときに自動でフィットし-その直後のノードラベルの文字高が11ピ.md) — satisfied
- [ac-6943 (AC-8) 現在どこを見ているかが画面に出ており、複数段戻ることができる](criteria/ac-6943-現在どこを見ているかが画面に出ており-複数段戻ることができる.md) — satisfied
- [ac-8544 (AC-29) 一覧と詳細でタイトル全文が読め、個別描画のラベルは折り返しで全文または先頭全角20文字以上が読める（実台帳で測る）](criteria/ac-8544-一覧と詳細でタイトル全文が読め-個別描画のラベルは折り返しで全文または先頭全角2.md) — satisfied
- [ac-8d4d (AC-33) location.spec の履歴復元テストが単独 10 回連続と全件 3 回連続で成功し、goBack 直後の hash と表示状態の一致が待機に依存しない形で検査される](criteria/ac-8d4d-location-spec-の履歴復元テストが単独-10-回連続と全件-3-回連.md) — satisfied
- [ac-92e9 (AC-1) 個別描画で、辺の長さの中央値を最近傍距離の中央値で割った値が5未満である](criteria/ac-92e9-個別描画で-辺の長さの中央値を最近傍距離の中央値で割った値が5未満である.md) — satisfied
- [ac-9e06 (AC-32) クリックハンドラを持つ全要素が button/link の役割とポインタカーソルを持ち、Overview の数値カードはそれぞれの一覧へ至るか枠を持たず、ポインタカーソルを持つのに何も起きない要素が 0 件である](criteria/ac-9e06-クリックハンドラを持つ全要素が-button-link-の役割とポインタカーソル.md) — satisfied
- [ac-a6d0 (AC-28) 詳細の本文で GFM の表・入れ子リスト・コードブロック・リンクが台帳本文どおりに描かれ、埋め込みエスケープの既存テストが通る](criteria/ac-a6d0-詳細の本文で-GFM-の表-入れ子リスト-コードブロック-リンクが台帳本文どおり.md) — satisfied
- [ac-b0f1 (AC-11) 型・スコープ・状態・充足・閉じ方・置き換え済みが、生のキーと値の行ではなくバッジまたはラベルとして描かれる](criteria/ac-b0f1-型-スコープ-状態-充足-閉じ方-置き換え済みが-生のキーと値の行ではなくバッジ.md) — satisfied
- [ac-b77a (AC-26) 検索欄と種別・スコープ・状態の絞り込みがヘッダー直下に常に見え、左パネルは Overview / Blockers / Progress だけになる](criteria/ac-b77a-検索欄と種別-スコープ-状態の絞り込みがヘッダー直下に常に見え-左パネルは-Ov.md) — satisfied
- [ac-bd5c (AC-4) 描画件数と非表示件数が、俯瞰と個別の両モードでバナーに出る](criteria/ac-bd5c-描画件数と非表示件数が-俯瞰と個別の両モードでバナーに出る.md) — satisfied
- [ac-d108 (AC-12) 成立範囲と本文が宣言の一覧と別の領域にあり、1行の文字数が日本語は全角30から45、欧文は45から90に収まる](criteria/ac-d108-成立範囲と本文が宣言の一覧と別の領域にあり-1行の文字数が日本語は全角30から4.md) — satisfied
- [ac-d906 (AC-30) 画面のソースが ui/ 配下の関心ごとのモジュールにあり、グローバル変数の共有が無く、bun build の成果物がコミット済みで CI が再現一致を検査し、再構成の前後で e2e の全テストが通る](criteria/ac-d906-画面のソースが-ui-配下の関心ごとのモジュールにあり-グローバル変数の共有が無.md) — satisfied
- [ac-e850 (AC-27) 種別・スコープ・状態で絞ると結果が一覧（表）として出て、クラスタや種別のクリックはその一覧へ至り、各行から1クリックで詳細が開く](criteria/ac-e850-種別-スコープ-状態で絞ると結果が一覧-表-として出て-クラスタや種別のクリック.md) — satisfied

## 履歴

| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |
|---|---|---|---|---|---|---|
| 1 | 2026-09-15 04:49 | lead | ac-92e9 (AC-1) 個別描画で、辺の長さの中央値を最近傍距離の中央値で割った値が5未満である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-577d (AC-10) 詳細を開いたとき、詳細領域の幅がグラフ領域の幅の0.9倍から1.1倍の範囲に入る（広い画面） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-b0f1 (AC-11) 型・スコープ・状態・充足・閉じ方・置き換え済みが、生のキーと値の行ではなくバッジまたはラベルとして描かれる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-d108 (AC-12) 成立範囲と本文が宣言の一覧と別の領域にあり、1行の文字数が日本語は全角30から45、欧文は45から90に収まる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-57a4 (AC-13) 未知属性が既知の宣言と分けた領域に残り、値が省略されない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-1967 (AC-2) 俯瞰から個別描画へ至る経路が、ノードIDを知らない利用者の操作だけで成立する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-5b53 (AC-24) gy.html#<ID> を開くとそのノードの詳細が開いており、焦点・選択・絞り込み・検索語の変更が hash に反映され、ブラウザの戻るで直前の表示状態に戻る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-3e7f (AC-25) ノードをクリックしてもグラフの配置と焦点は変わらず詳細だけが開き、焦点移動と絞り込みは押す前に結果が読める明示ボタンで行われる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-b77a (AC-26) 検索欄と種別・スコープ・状態の絞り込みがヘッダー直下に常に見え、左パネルは Overview / Blockers / Progress だけになる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-e850 (AC-27) 種別・スコープ・状態で絞ると結果が一覧（表）として出て、クラスタや種別のクリックはその一覧へ至り、各行から1クリックで詳細が開く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-a6d0 (AC-28) 詳細の本文で GFM の表・入れ子リスト・コードブロック・リンクが台帳本文どおりに描かれ、埋め込みエスケープの既存テストが通る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-8544 (AC-29) 一覧と詳細でタイトル全文が読め、個別描画のラベルは折り返しで全文または先頭全角20文字以上が読める（実台帳で測る） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-670c (AC-3) 表示対象が変わったときに自動でフィットし、その直後のノードラベルの文字高が11ピクセル以上である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-d906 (AC-30) 画面のソースが ui/ 配下の関心ごとのモジュールにあり、グローバル変数の共有が無く、bun build の成果物がコミット済みで CI が再現一致を検査し、再構成の前後で e2e の全テストが通る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-0b0b (AC-31) ツールバーが絞り込み・グラフ操作・リセットの3群に分かれて 1440px で 1 行に収まり、状態の表示（焦点・件数・絞り込み）が画面に 1 か所だけあり、凡例がグラフのノードを覆わない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-9e06 (AC-32) クリックハンドラを持つ全要素が button/link の役割とポインタカーソルを持ち、Overview の数値カードはそれぞれの一覧へ至るか枠を持たず、ポインタカーソルを持つのに何も起きない要素が 0 件である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-8d4d (AC-33) location.spec の履歴復元テストが単独 10 回連続と全件 3 回連続で成功し、goBack 直後の hash と表示状態の一致が待機に依存しない形で検査される | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-4847 (AC-34) 個別描画で辺ラベルの矩形がノードのタイトル矩形と交差せず、交差する場合は辺ラベルを出さずに選択した辺だけで表示する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-bd5c (AC-4) 描画件数と非表示件数が、俯瞰と個別の両モードでバナーに出る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-1dcb (AC-5) 実台帳314ノードで、生成したHTMLがネットワークへ出ずに file:// で全機能動作する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-1da3 (AC-6) 分割の前後で、同じ台帳から生成されるHTMLが同一である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-1fe0 (AC-7) 個別描画のノードをクリックすると焦点がそのノードへ移り、直前の焦点へ戻せる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-6943 (AC-8) 現在どこを見ているかが画面に出ており、複数段戻ることができる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-4392 (AC-9) 自動化されたE2Eが、クリックの到達と表示対象の変化に伴う再フィットを検出する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-3755 (D-1) HTML投影の入口は render --format html とする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-521b (D-10) 近傍展開のホップ数は起点ごとに閾値へ収まる最大値を選ぶ | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-51f6 (D-11) init が gy.toml を正規化して書き直す挙動は変えず、文書で伝える | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-9ad5 (D-12) ズーム段階に連動した3段階の詳細度でグラフを描き分ける | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-ce87 (D-13) gy 自身の台帳はバージョン管理へ入れず、AGENTS.md に運用を定める | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-3c53 (D-14) 浮上と降下は、焦点と半径だけを持つモードレスな構成で実現する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-2c8a (D-15) template.html の分割をナビゲーションの修正より前に行う | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-d764 (D-16) HTMLの振る舞いの検証に自動化されたE2Eを導入する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-29e7 (D-17) 再フィットは現在の変換の由来で分け、1回の操作につき1回に限る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-6793 (D-18) 個別描画の自動フィットは倍率2を上限とし、手動ズームの上限4を維持する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-73a5 (D-19) 孤立した焦点にはグラフ上の接続が無いことを画面に示す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-2455 (D-2) 生成したHTMLはローカル生成のみとし、init が .gitignore へ追記する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-abe1 (D-20) 自動表示は可読性を優先し、焦点または表示対象の中心を基準に画面外件数を示す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-6acb (D-21) 焦点があるときは焦点を含む最大件数まで個別描画し、残りの件数と一覧への導線を示す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-42c9 (D-22) E2E と動作保証の対象は Chromium だけとする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-9270 (D-23) 詳細は宣言と説明を分けた読み物として構成し、グラフ領域と同等の幅を持たせる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-ba72 (D-24) AC-12 の行長は文字体系ごとの慣用に合わせ、日本語は全角30から45文字、欧文は45から90文字とする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-0621 (D-25) HTMLの自己完結性は読込時と描画操作時に外部取得を行わないことで検証する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-a1a2 (D-26) CI は Linux のみで回し、自動操作の往復時間を関門にしない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-84bc (D-3) ノード本文はHTMLへ全文埋め込む | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-5d63 (D-37) HTML projection が支える業務は、指名到達・絞り込んで一覧を読む・読む・辿る・状況把握の5つである | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-626e (D-38) 場所は URL が持つ: 表示状態は hash に載り、ブラウザの戻るが効き、ID を貼れば開く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-4425 (D-39) モードレス: 同じ操作は状態に依らず同じ意味を持ち、クリックは選んで見る、配置や絞り込みの変更は明示操作である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-f45a (D-4) non_exhaustive は公開設定型9種すべてに付ける | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-1a18 (D-40) 一覧が主、グラフは関係を見る補助である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-3026 (D-41) 検索と絞り込みは常設のツールバーとしてヘッダー直下に置く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-47e7 (D-42) 本文は台帳の Markdown を欠けずに描く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-c6bd (D-43) タイトルは全文が読める場所を必ず持ち、グラフのラベルは折り返しを優先して省略を最後の手段とする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-6e4d (D-44) 押せるものは押せる見た目を持ち、押せる見た目のものは押すと何かが起きる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-a2c6 (D-45) 画面のコードは関心ごとの ES モジュールと TypeScript で書き、bun で 1 本に束ねた成果物を html.rs が埋め込む | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-c567 (D-46) CI に追加した検査は、同一コマンドのローカル成功と欠陥注入での失敗を根拠に受け入れ、GitHub 上の実行結果は push 後に追補の evidence として確かめる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-706f (D-47) hash は表示の指定を保持し、データは開いた HTML から導く。出所付きの一覧は種別化した出所（overview=next など）で表し、ID 集合は保存しない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-6380 (D-48) 一覧の行は表の row 役割を保ち、行全体を覆う native link を実ヒット対象とする。押せる見た目の検査は要素の計算スタイルではなく実ヒット対象で行う | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-5766 (D-49) ラベルの全収容（文字高 11px 以上・交差 0・画面外 0）は通常の個別描画に適用し、Lineage は折り返しラベルと世代配置を保ち、可読倍率とパンで読む | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-affc (D-5) RecordSchema と FieldSchema と RecordCheck は読み取り専用型とし Default を実装しない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-0325 (D-50) 辺ラベルはノードのタイトルと衝突するとき隠し、関係はノード詳細の関係一覧で読む。辺の選択という操作状態は設けない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-36b2 (D-6) lint の結果は render の終了コードを変えない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-b9ff (D-7) 0.3.0 はドッグフーディングを待たずに公開する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-d97b (D-8) グラフの描き分けはズーム段階ではなく表示対象の件数で行う | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-4aa9 (D-80) html_projection の未着手ニーズ 14 件は D-79 により実装せずに閉じ、status を complete にして dropped_by で D-80 を指す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-a3e2 (D-9) 近傍展開の辺集合は画面が描く EDGES ベースとする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-76b4 (N-1) 集約辺のラベルを、対応する曲線の近くへ置く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-c3b4 (N-10) 成立範囲と本文を読める行長の読み物として配置する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-afe9 (N-19) 表示状態を URL の hash に載せ、ID で直接到達でき、戻るが効くようにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-8c5e (N-2) フィットの計算にラベルの広がりを含め、端でラベルが切れないようにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-f025 (N-20) ノードのクリックを詳細表示だけにし、焦点移動と絞り込みを明示操作にする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-ac8e (N-21) 検索と絞り込みをヘッダー直下の常設ツールバーにし、左パネルを状況把握に限る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-ca04 (N-22) 絞り込み結果の一覧ビューを置き、クラスタと種別のクリックを一覧へ導く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-cda9 (N-23) 詳細本文の Markdown 描画を GFM の表・入れ子リスト・コード・リンクに対応させる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-4759 (N-24) タイトルを一覧と詳細で全文、グラフのラベルで折り返し表示にする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-b4d3 (N-25) 画面のコードを ui/ 配下の TypeScript モジュールに再構成し、bun で束ねた成果物を html.rs が埋め込む形にする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-3f2d (N-26) ツールバーと状態表示を設計トークンに基づいて組み直し、絞り込み・グラフ操作・リセットを分ける | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-1b72 (N-27) 押せる見た目と押せる実体を一致させ、Overview の数値カードを一覧への入口にする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-5ba8 (N-28) 履歴の戻る/進む直後に hash と表示状態が一致することを、時間依存なく成立させる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-cb5e (N-29) 辺ラベルがノードのタイトルと重ならないようにし、重なる場合は選択した辺だけに出す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-7170 (N-3) バナーを1つに統合し、俯瞰では clusters、個別では nodes と書き分ける | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-afb6 (N-5) 焦点を履歴のあるスタックにし、ノードのクリックを焦点の移動とする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-0228 (N-6) 現在の焦点までの経路を画面に出し、任意の段へ戻せるようにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-8619 (N-7) playwright 相当のE2Eを開発専用として導入し、CI から実行できるようにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-2e91 (N-8) 詳細パネルをグラフ領域と同等の幅にする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-f20e (N-9) メタ情報をバッジとラベルで構造化する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-8aca (Q-1) 台帳をHTMLへ投影する入口は render の追加オプションか、新しいコマンドか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-ab52 (Q-10) 近傍展開のホップ数を固定するか、起点ごとに可変にするか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-4dd9 (Q-11) init が既存の gy.toml を正規化して書き直す挙動を変えるか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-83d4 (Q-12) gy 自身の台帳をバージョン管理へ入れるか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-231e (Q-13) グラフの浮上と降下を、モードの切り替えで作るかモードレスな焦点の移動で作るか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-041d (Q-14) template.html の分割を、ナビゲーションの修正より前に行うか後に行うか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-735e (Q-15) HTMLの振る舞いの検証に何を使うか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-1dbb (Q-16) 描画領域の大きさだけが変わったときに再フィットするか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-281b (Q-17) 焦点の1ホップが60件を超えるときクラスタへ戻すか個別表示を継続するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-fd63 (Q-18) E2E と動作保証の対象ブラウザをいくつにするか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-4b50 (Q-19) 要求にしないまま終わったニーズを next から外す手段が無い | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-32af (Q-2) 生成したHTMLをバージョン管理へ入れるか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-54ea (Q-20) 詳細パネルを読み物として成立させるか、属性の一覧として保つか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-1aa0 (Q-21) 詳細の等幅配置でAC-12の行長下限をどの画面幅と文字幅に適用するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-16f6 (Q-22) CI の対象プラットフォームと性能の関門をどう置くか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-6fbc (Q-3) ノード本文をHTMLへ全文埋め込むか、要約して遅延読込にするか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-6838 (Q-30) N-25 の CI 再現検査はローカル実行の結果で受け入れるか、GitHub 実行まで待つか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-6d40 (Q-31) Overviewから選ぶnext集合をURLで意味として保存するかID集合として保存するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-4473 (Q-32) 常設表の行リンク検査は実際のヒット対象で行うかtrをlinkへ変更するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-5921 (Q-33) N-24の全ラベル画面内収容はLineageの世代配置にも適用するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-e833 (Q-34) N-29 の選択辺フォールバックは新設するか、既存のノード詳細へ委ねるか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-103c (Q-4) non_exhaustive を RenderConfig だけに付けるか、公開設定型すべてに付けるか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-a09e (Q-5) Default を持たない RecordSchema / FieldSchema / RecordCheck に Default を足すか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-7234 (Q-6) render の終了コードは lint の結果を反映すべきか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-83bf (Q-7) 0.3.0 をドッグフーディングのフィードバックより前に公開するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-1eb0 (Q-8) グラフの描き分けはズーム段階で切り替えるか、表示対象の件数で切り替えるか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-3d01 (Q-9) 近傍展開が使う辺集合は、画面が描く EDGES か、frontmatter の全参照か | created | migrate from 0.4 | publication/0.4 0d42e55 |

## 診断

エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。警告は注意が要る状態で、`handover` に一覧が出る。

errors: 0
warnings: 1
- criteria with an empty body: 24
