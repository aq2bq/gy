# gy の公開物 — gy05

- 生成: 2026-09-15T10:22:54Z
- seq: 158
- scope: gy05
- since: (先頭から)
- 書き手: lead
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
- [n-0471 (N-36) bearer_count と need の状態を導出にし、handover を error と進行中だけにする](needs/n-0471-bearer_count-と-need-の状態を導出にし-handover-を.md) — closed
- [n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ](needs/n-0dae-正本を-git-の外に置き-ハッシュ-ID-GY_ACTOR-必須の履歴-und.md) — closed
- [n-271d (N-43) 文書と同梱 skill を新しい gy に合わせて書き直す（README 日英、architecture、CHEATSHEET、skill）](needs/n-271d-文書と同梱-skill-を新しい-gy-に合わせて書き直す-README-日英.md) — closed
- [n-2f21 (N-40) publish (a): views/publish の骨格と「いま判断待ち」（問い 2）、「未決の論点」（問い 6）、「注意」の節](needs/n-2f21-publish-a-views-publish-の骨格と-いま判断待ち-問い-2.md) — closed
- [n-3d8c (N-35) workflow.records / guards / スナップショット / lint 設定 / import 設定 / 申告オプション / gate を消し、gy.toml をスコープと出力先だけにする](needs/n-3d8c-workflow-records-guards-スナップショット-lint-設定.md) — closed
- [n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）](needs/n-4fe0-gy-自身の台帳と-Kokopelli-の-0-4-台帳を一回で移行する-自分の.md) — closed
- [n-66ee (N-34) 要求の状態を 4 つにし、確定後は改訂・完了・中止の記録だけを持ち、ID を振り直して ref を持たせる](needs/n-66ee-要求の状態を-4-つにし-確定後は改訂-完了-中止の記録だけを持ち-ID-を振り.md) — closed
- [n-bc0c (N-42) 新しい core の骨格を 4 層と型のモデルで作り、行数・glob・文字列キー・差分の大きさを測る道具を用意する](needs/n-bc0c-新しい-core-の骨格を-4-層と型のモデルで作り-行数-glob-文字列キー.md) — closed
- [n-da96 (N-39) 各コマンドの出力が、そのノードに無いものと次の操作を返し、同梱 skill を流れの説明だけにする](needs/n-da96-各コマンドの出力が-そのノードに無いものと次の操作を返し-同梱-skill-を流.md) — closed
- [n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い）](needs/n-dbcf-操作を末端-19-の閉じた集合にし-一つの意図を一つのコマンド-一つのトランザク.md) — closed
- [n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）](needs/n-00dc-移行-b1-辺と-waits-on-Relation-に-WaitsOn-nex.md) — closed
- [n-02b9 WebUI（第 3 段）: マスターが「これは何で何と関係があるか」「今マスター待ちは何か」を把握できる画面。問いは D-85 の 6 つを入力にし、作るのはマスターの指示があってから](needs/n-02b9-WebUI-第-3-段-マスターが-これは何で何と関係があるか-今マスター待ちは.md) — closed
- [n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割）](needs/n-0476-読み-2b-handover-error-と進行中の要求を-ref-付きで-wa.md) — closed
- [n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）](needs/n-0551-移行-a1-クレート-gy-migrate-の骨格-引数-ガード-0-4-を-g.md) — closed
- [n-0cb7 (N-48) 正本の置き場所（XDG のデータディレクトリ + リポジトリのハッシュ）と形式の版 format の読み書き・判定・移行の入口の枠](needs/n-0cb7-正本の置き場所-XDG-のデータディレクトリ-リポジトリのハッシュ-と形式の版.md) — closed
- [n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示）](needs/n-147b-0-5-0-の準備-gy5-を-gy-に改名し-gy-core-と-0-4-の.md) — closed
- [n-1aad edit --set scope=<名前> でノードを別のスコープへ移せるようにする（名前は gy.toml にあるものだけ。ID と辺は変わらない）](needs/n-1aad-edit---set-scope-名前-でノードを別のスコープへ移せるようにする.md) — closed
- [n-24dd scope rename (c): CLI と gy.toml の書き換え（コメント・順序を保ち、失敗時はログを戻す）、文書、gy 自身の写しでの確認（n-ff2b から分割）](needs/n-24dd-scope-rename-c-CLI-と-gy-toml-の書き換え-コメント.md) — closed
- [n-348d (N-45) 物理設計の基準を機械で測る道具 scripts/measure.sh を用意する（行数・関数の長さ・glob・文字列キー・テストの行数・差分・層の向き）](needs/n-348d-物理設計の基準を機械で測る道具-scripts-measure-sh-を用意する.md) — closed
- [n-35cf show / list / publish のニーズの状態を next と同じ導出（open / closed / done）にする。list --status done を受ける](needs/n-35cf-show-list-publish-のニーズの状態を-next-と同じ導出-op.md) — closed
- [n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割）](needs/n-3806-移行-a2-ノードの写し-種類ごとの対応表-別名-ref-1-トランザクション.md) — closed
- [n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割）](needs/n-3a5d-移行-b3-記録の凍結-publication-と-legacy_records.md) — closed
- [n-3f44 (N-44) render と HTML 投影（html.rs、ui/、e2e/、テスト、同梱物、CI）を最初に消し、しがらみの無い木で始める](needs/n-3f44-render-と-HTML-投影-html-rs-ui-e2e-テスト-同梱物.md) — closed
- [n-3f84 scope rename (b): 操作 scope_rename（Repository 経由の適用、履歴 1 行、undo、Outcome）とテスト（n-ff2b から分割）](needs/n-3f84-scope-rename-b-操作-scope_rename-Repositor.md) — closed
- [n-447c (N-54) 操作 (d): edit（--title / --body / --set / --append、--reason 必須。状態と辺は変えない）と undo の操作](needs/n-447c-操作-d-edit---title---body---set---append.md) — closed
- [n-45ef publish (a): スコープごとのディレクトリ出力と 1 ノード 1 ファイル（関係と mark を相手の題名付きで。対象スコープのディレクトリだけを作り直す）](needs/n-45ef-publish-a-スコープごとのディレクトリ出力と-1-ノード-1-ファイル.md) — closed
- [n-478b (N-64) CLI (2b): req add / approve / revise / done / cancel の配線と末端の数の測定（AC-49）（N-62 から分割）](needs/n-478b-CLI-2b-req-add-approve-revise-done-cance.md) — closed
- [n-5091 (N-71) publish (b): ノードの節（問い 1・3・4）と「この期間の変更」（問い 5）、gy5 の配線、publish-sample.md（N-40 から分割）](needs/n-5091-publish-b-ノードの節-問い-1-3-4-と-この期間の変更-問い-5.md) — closed
- [n-54fe (N-61) CLI (1a): 新しい gy の CLI クレート gy5 の骨格（clap、GY_ACTOR、--json、-C、終了コード、正本の自動作成）と読み 4 の配線](needs/n-54fe-CLI-1a-新しい-gy-の-CLI-クレート-gy5-の骨格-clap-GY.md) — closed
- [n-58b4 (N-58) 読み (1b): list（--type / --status / --targets / --grep / --actor / --since。actor と since は書き込み単位）を views 層に作る（N-57 から分割）](needs/n-58b4-読み-1b-list---type---status---targets---g.md) — closed
- [n-5c59 publish (b): スコープごとの索引 README.md（見出し・読み方・一覧とリンク・履歴・診断）と --since、gy.toml の output、文書、gy 自身の docs/publication の作り直し（n-45ef から分割）](needs/n-5c59-publish-b-スコープごとの索引-README-md-見出し-読み方-一覧.md) — closed
- [n-6eae publish <ID> を見出しと箇条書きの形にし、既定の読み物の見本を 2 つ（判断待ちが無いとき / あるとき）作ってマスターに見せる](needs/n-6eae-publish-ID-を見出しと箇条書きの形にし-既定の読み物の見本を-2-つ.md) — closed
- [n-79fb edit で、未記録の成立範囲だけを 1 回記録できる（--set decision_scope=<文>。記録済みは拒む）。--set key= は自由属性を消す](needs/n-79fb-edit-で-未記録の成立範囲だけを-1-回記録できる---set-decisi.md) — closed
- [n-7cc1 (N-57) 読み (1a): views の骨格と show（複数 ID、ref でも引ける、種類ごとの逐語、--full、--json）](needs/n-7cc1-読み-1a-views-の骨格と-show-複数-ID-ref-でも引ける-種類.md) — closed
- [n-816a (N-55) 操作 (a2): question add / question close / criterion add / criterion satisfy の 4 操作（N-51 から分割）](needs/n-816a-操作-a2-question-add-question-close-criter.md) — closed
- [n-81d8 (N-52) 操作 (b): decide（ADR 有り無し、closes / narrows / supersedes / completes / widens と mark）と link（--mark、--remove）](needs/n-81d8-操作-b-decide-ADR-有り無し-closes-narrows-supe.md) — closed
- [n-8996 (N-59) 読み (2a): HistoryEntry の seq、need の状態の導出、next を views 層に作る](needs/n-8996-読み-2a-HistoryEntry-の-seq-need-の状態の導出-nex.md) — closed
- [n-8f3f publish を記録の公開物にする: 指定した時点と範囲のノード・辺・履歴と診断結果を 1 ファイルに出し、--out（gy.toml の output）へ書いてコミットできる形にする。1 ページ化（N-72）は戻す](needs/n-8f3f-publish-を記録の公開物にする-指定した時点と範囲のノード-辺-履歴と診断.md) — closed
- [n-9198 gy-migrate が既知の名前以外の自由属性を落とし報告にも出さない欠陥を直す: すべての残った属性を自由属性（配列は改行区切り）として写し、報告に一覧を出す。edit --append の意味（文字列に 1 行足す）を文書に明記](needs/n-9198-gy-migrate-が既知の名前以外の自由属性を落とし報告にも出さない欠陥を直.md) — closed
- [n-a603 (N-46) 新しい core の model 層: 5 種のノードの型と不変条件を src/model/ のディレクトリ分割で作る](needs/n-a603-新しい-core-の-model-層-5-種のノードの型と不変条件を-src-m.md) — closed
- [n-b6a7 waits-on の先に要求を許す: model の組、derive の準備判定、gy-migrate の waiting-on の写し、テスト](needs/n-b6a7-waits-on-の先に要求を許す-model-の組-derive-の準備判定.md) — closed
- [n-b6f3 (N-72) publish を 1 ページの読み物に直す: 既定はいま判断待ち・未決・注意の件数・期間の変更だけ、ノードの記述は publish <ID>... で指定した分だけ](needs/n-b6f3-publish-を-1-ページの読み物に直す-既定はいま判断待ち-未決-注意の件.md) — closed
- [n-bd8d (N-49) undo（逆変更の行の追記）と snapshot.json（commit ごとの原子的な書き直し、無くてもログの再生で復元）](needs/n-bd8d-undo-逆変更の行の追記-と-snapshot-json-commit-ごとの.md) — closed
- [n-c980 (N-56) 操作 (c2): req approve（--design、--heard-by、--evidence）/ revise / done / cancel と、確定後の 3 つの記録（N-53 から分割）](needs/n-c980-操作-c2-req-approve---design---heard-by.md) — closed
- [n-d1bf (N-50) 操作の共通の枠: gy.toml（スコープと出力先だけ）の読み込み、型付きノードの読み書きと ID・別名の解決を担う Repository、1 意図 1 トランザクションの実行の型、出力の型（ID・変えた項目・無いもの・次の操作）](needs/n-d1bf-操作の共通の枠-gy-toml-スコープと出力先だけ-の読み込み-型付きノードの.md) — closed
- [n-d1c6 (N-53) 操作 (c1): req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）と、resolve の ref 対応](needs/n-d1c6-操作-c1-req-add---need-複数---relies-on---ta.md) — closed
- [n-d514 (N-51) 操作 (a1): 辺を from 側だけに置き Need.targets の重複を消し、need add / need close を作る](needs/n-d514-操作-a1-辺を-from-側だけに置き-Need-targets-の重複を消し.md) — closed
- [n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正](needs/n-e3c1-移行-b2-legacy-rs-の分割-要求の-11-状態-4-状態と-appr.md) — closed
- [n-edce (N-47) 辺の種類の組（論点 closes 決定、ニーズ targets 受け入れ条件など）を model の不変条件として検査し、外れた組の Link を作れなくする](needs/n-edce-辺の種類の組-論点-closes-決定-ニーズ-targets-受け入れ条件など.md) — closed
- [n-ef16 publish のノードのファイルの形を整える: 関係は 1 か所（両向き）、成立範囲は 1 回、節の間に空行、書き手が無ければ行を出さない](needs/n-ef16-publish-のノードのファイルの形を整える-関係は-1-か所-両向き-成立範.md) — closed
- [n-f088 ID の解決で、0 埋めの同一視を旧 ID の別名にだけ当て、gy が振ったハッシュ ID には当てない（d-0008 が D-8 と衝突する欠陥）。新しい ID は数字だけのハッシュを避ける](needs/n-f088-ID-の解決で-0-埋めの同一視を旧-ID-の別名にだけ当て-gy-が振ったハッ.md) — closed
- [n-f312 (N-62) CLI (2a): decide / link / edit / undo の配線と、書きの入口での actor 検査の前倒し](needs/n-f312-CLI-2a-decide-link-edit-undo-の配線と-書きの入口で.md) — closed
- [n-f65b (N-63) CLI (1b): need / question / criterion の 6 操作の配線とテスト（N-61 から分割）](needs/n-f65b-CLI-1b-need-question-criterion-の-6-操作の配線.md) — closed
- [n-ff2b scope rename (a): 形式の版 2 と写し、ログの変更の種類 scope-renamed と再生](needs/n-ff2b-scope-rename-a-形式の版-2-と写し-ログの変更の種類-scope.md) — closed
### questions
- [q-037e (Q-42) publish の読み物が答えるマスターの問いは何か（roadmap 未決 2、第 3 段の入力。問いが揃うまで作らない）](questions/q-037e-publish-の読み物が答えるマスターの問いは何か-roadmap-未決-2.md) — closed
- [q-6c8e (Q-41) ニーズの waiting-on（待っている論点）は自由属性の文字列のままでよいか、閉じた関係（need waits-on question）にして書き込み時に検証するか](questions/q-6c8e-ニーズの-waiting-on-待っている論点-は自由属性の文字列のままでよいか.md) — closed
- [q-875f (Q-40) 末端と固有オプションの数: undo と publish を含めると末端 20、固有オプションは distinct 30 前後になる。AC-49「末端 19 以下、固有オプション 20 前後」をどう扱うか](questions/q-875f-末端と固有オプションの数-undo-と-publish-を含めると末端-20-固.md) — closed
- [q-c5fc (Q-43) 新しい gy の版番号は 0.5.0 か 1.0.0 か（roadmap 未決 3）](questions/q-c5fc-新しい-gy-の版番号は-0-5-0-か-1-0-0-か-roadmap-未決.md) — closed
- [q-c7b2 (Q-39) 新しい gy の正本の保存形式は何か（N-38 の前提。D-61 の要件から導く）](questions/q-c7b2-新しい-gy-の正本の保存形式は何か-N-38-の前提-D-61-の要件から導く.md) — closed
### decisions
- [d-b108 (D-78) N-42 の契約: 新しい core はクレート gy-ledger に 4 層で作り、永続化の実体は持たず、測る道具は scripts/measure.sh に置く](decisions/d-b108-N-42-の契約-新しい-core-はクレート-gy-ledger-に-4-層で.md)
- [d-1cfb (D-84) 新しい gy の操作の集合は末端 20（書き 15、読み 5）、固有オプション 30 以下とし、AC-49 をこの数に改める](decisions/d-1cfb-新しい-gy-の操作の集合は末端-20-書き-15-読み-5-固有オプション-3.md)
- [d-38b5 publish <ID> の答えは「見出しと箇条書き」の形にする: 見出しは旧 ID と短い題、節は 決めたこと / 理由 / 生んだ作業（状態付き）/ 状態（有効か、置き換えの有無）。要求は 来歴 / 状態 / 記録、ニーズは 目的 / 受け入れ条件 / 起票した要求](decisions/d-38b5-publish-ID-の答えは-見出しと箇条書き-の形にする-見出しは旧-ID.md)
- [d-63f8 waits-on の先には論点に加えて要求も許し、要求が Done か Cancelled になれば待ちが解ける](decisions/d-63f8-waits-on-の先には論点に加えて要求も許し-要求が-Done-か-Canc.md)
- [d-648a (D-87) publish の既定は 1 ページの読み物（いま判断待ち、未決の論点、注意の件数、期間の変更）とし、ノードの記述（問い 1・3・4）は指定した ID の分だけ出す。全ノードの書き出しはしない](decisions/d-648a-publish-の既定は-1-ページの読み物-いま判断待ち-未決の論点-注意の件.md)
- [d-6e70 スコープ名の変更を操作 scope rename として閉じた集合に足す。履歴には「スコープ名の変更 旧 → 新（n ノード）」の 1 件の変更として残し、gy が gy.toml の [scopes.旧] を [scopes.新] に書き換える](decisions/d-6e70-スコープ名の変更を操作-scope-rename-として閉じた集合に足す-履歴に.md)
- [d-736e 0.4 系のスコープに残る未着手ニーズは、消した機能に伴うものと新しい gy が別の形で満たしたものに分けて閉じる。WebUI のニーズはマスターの指示があるまで立てない](decisions/d-736e-0-4-系のスコープに残る未着手ニーズは-消した機能に伴うものと新しい-gy-が.md)
- [d-799e (D-85) publish の読み物は 6 つの問いに答える形で作り、マスターが読んで答えられなかった問いを後から足す](decisions/d-799e-publish-の読み物は-6-つの問いに答える形で作り-マスターが読んで答えら.md)
- [d-7c64 publish の出力は 1 ファイルでなく、スコープごとのディレクトリに 1 ノード 1 ファイルと索引 1 ファイルを書き出す形にし、出力先のスコープのディレクトリの中だけを作り直す](decisions/d-7c64-publish-の出力は-1-ファイルでなく-スコープごとのディレクトリに-1.md)
- [d-8637 (D-83) ニーズが待っている論点は閉じた関係 waits-on（need → question）で持ち、書き込み時に検証する。自由属性 waiting-on は移行で辺に写す](decisions/d-8637-ニーズが待っている論点は閉じた関係-waits-on-need-question.md)
- [d-9751 (D-82) 新しい gy の正本は追記専用の JSONL イベントログ + スナップショット（ファイル、依存ゼロ）で持つ](decisions/d-9751-新しい-gy-の正本は追記専用の-JSONL-イベントログ-スナップショット-フ.md)
- [d-9764 (D-81) N-44 の契約: render サブコマンドと HTML 投影を丸ごと消し、gy.toml の [render] は N-35 まで読み飛ばし、差分の上限は追加・変更行だけに掛ける](decisions/d-9764-N-44-の契約-render-サブコマンドと-HTML-投影を丸ごと消し-gy.md)
- [d-d205 (D-86) 新しい gy の版は 0.5.0 とし、非互換の変更を一つの版にまとめる。1.0.0 は Kokopelli の実運用の移行と AC-51 / AC-52 の測定が済み publish の問いが安定した時点で改めて問う](decisions/d-d205-新しい-gy-の版は-0-5-0-とし-非互換の変更を一つの版にまとめる-1-0.md)
- [d-dcbb 0.4 の運用を決めた古い決定は消さず、新しい決定で置き換える（supersedes と mark）。公開物にも「置き換えられた」として残す。正本からノードを消す操作は提供しない](decisions/d-dcbb-0-4-の運用を決めた古い決定は消さず-新しい決定で置き換える-supersed.md)
- [d-edb0 publish は開発の成果物としてコミットし後から過去の判断と経緯を振り返るための記録であり、開発中にマスターが読むものではない。「D-78 って何だっけ」に答えるのは WebUI のニーズで、gy のコマンドには人間向けの出力の形を足さない](decisions/d-edb0-publish-は開発の成果物としてコミットし後から過去の判断と経緯を振り返るた.md)
### requirements
- 無し
### criteria
- [ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける](criteria/ac-09b1-末端のサブコマンドが-21-以下-書き-16-読み-5-固有オプションが-30.md) — satisfied
- [ac-5216 (AC-41) 確定後の転記が 0。進行管理が外の文書から gy へ写す記録が無い（今日は #6027 で約 22,800 bytes）](criteria/ac-5216-確定後の転記が-0-進行管理が外の文書から-gy-へ写す記録が無い-今日は-60.md) — unsatisfied
- [ac-6251 (AC-50) 利用者の文書と運用に採番の規則が現れない。ID の衝突と振り直しが起きない](criteria/ac-6251-利用者の文書と運用に採番の規則が現れない-ID-の衝突と振り直しが起きない.md) — satisfied
- [ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す](criteria/ac-7652-後から検査する-lint-が無く-不正な状態は書き込み時に拒まれ-注意が要る状態.md) — satisfied
- [ac-7670 (AC-45) 1 コマンドの書き込みは全部書けるか全部書かないか。途中失敗で中途半端な状態が残らない](criteria/ac-7670-1-コマンドの書き込みは全部書けるか全部書かないか-途中失敗で中途半端な状態が残.md) — satisfied
- [ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる](criteria/ac-8787-publish-の出力が-指定した時点と範囲の記録-ノード-辺-履歴-と診断結果.md) — satisfied
- [ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ](criteria/ac-8ca4-gy-のコマンド-設定-ID-項目名に特定の外部サービスの語が無く-外への参照は.md) — satisfied
- [ac-9be9 (AC-40) gy.toml に書けるのはスコープと出力先だけで、他のキーは読み込み時にエラー。Kokopelli の gy.toml 2,117 行が 16 行相当で読み込める](criteria/ac-9be9-gy-toml-に書けるのはスコープと出力先だけで-他のキーは読み込み時にエラー.md) — satisfied
- [ac-a8ff (AC-51) Kokopelli の TEAM_AGENTS.md の gy に関する行数が、gy の出力が次の操作と不足を返すことで減る](criteria/ac-a8ff-Kokopelli-の-TEAM_AGENTS-md-の-gy-に関する行数が.md) — satisfied
- [ac-b0f3 (AC-46) すべての書き込みに、いつ・誰が（GY_ACTOR 必須）・なぜ・出典が残り、undo できる](criteria/ac-b0f3-すべての書き込みに-いつ-誰が-GY_ACTOR-必須-なぜ-出典が残り-und.md) — satisfied
- [ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ](criteria/ac-c21e-セッション再開の-1-コマンド-handover-で-進行中と確定-未完了の要求.md) — satisfied
- [ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている](criteria/ac-c2bd-正本が形式の版を持ち-古い版を開くと写しを残して-1-トランザクションで移行され.md) — satisfied
- [ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない](criteria/ac-ce32-ソースリポジトリの作業ツリーに-gy-のファイルが無く-台帳のための-git-も.md) — satisfied
- [ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0](criteria/ac-d8f9-Kokopelli-の-0-4-台帳が一回の移行で入る-全ノードはハッシュ-ID.md) — satisfied
- [ac-d95a (AC-56) README 日英・docs・CHEATSHEET・同梱 skill が新しい gy の 20 の操作（書き 15、読み 5。D-84）と一つの進め方だけを説明し、消した設定・操作・規則への言及が無い（grep で 0）](criteria/ac-d95a-README-日英-docs-CHEATSHEET-同梱-skill-が新しい.md) — satisfied
- [ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている](criteria/ac-e8b7-すべての変更が物理設計の基準-1-ファイル-300-行-1-関数-40-行-gl.md) — satisfied
- [ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%）](criteria/ac-fc89-要求ノードは確定前の記述と辺と-ref-だけ-スナップショットが無い-今日は-4.md) — satisfied
- [ac-d2ab (AC-57) render サブコマンド、html.rs、ui/、e2e/、html_render のテスト、同梱の dist と template、CI の該当ジョブが無く、cargo test --workspace と cargo package --list にそれらが現れない](criteria/ac-d2ab-render-サブコマンド-html-rs-ui-e2e-html_render.md) — satisfied

## 履歴

| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |
|---|---|---|---|---|---|---|
| 1 | 2026-09-15 04:49 | lead | ac-9be9 (AC-40) gy.toml に書けるのはスコープと出力先だけで、他のキーは読み込み時にエラー。Kokopelli の gy.toml 2,117 行が 16 行相当で読み込める | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-5216 (AC-41) 確定後の転記が 0。進行管理が外の文書から gy へ写す記録が無い（今日は #6027 で約 22,800 bytes） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-7670 (AC-45) 1 コマンドの書き込みは全部書けるか全部書かないか。途中失敗で中途半端な状態が残らない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-b0f3 (AC-46) すべての書き込みに、いつ・誰が（GY_ACTOR 必須）・なぜ・出典が残り、undo できる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-6251 (AC-50) 利用者の文書と運用に採番の規則が現れない。ID の衝突と振り直しが起きない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-a8ff (AC-51) Kokopelli の TEAM_AGENTS.md の gy に関する行数が、gy の出力が次の操作と不足を返すことで減る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-d95a (AC-56) README 日英・docs・CHEATSHEET・同梱 skill が新しい gy の 20 の操作（書き 15、読み 5。D-84）と一つの進め方だけを説明し、消した設定・操作・規則への言及が無い（grep で 0） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-d2ab (AC-57) render サブコマンド、html.rs、ui/、e2e/、html_render のテスト、同梱の dist と template、CI の該当ジョブが無く、cargo test --workspace と cargo package --list にそれらが現れない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-b108 (D-78) N-42 の契約: 新しい core はクレート gy-ledger に 4 層で作り、永続化の実体は持たず、測る道具は scripts/measure.sh に置く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-9764 (D-81) N-44 の契約: render サブコマンドと HTML 投影を丸ごと消し、gy.toml の [render] は N-35 まで読み飛ばし、差分の上限は追加・変更行だけに掛ける | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-9751 (D-82) 新しい gy の正本は追記専用の JSONL イベントログ + スナップショット（ファイル、依存ゼロ）で持つ | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-8637 (D-83) ニーズが待っている論点は閉じた関係 waits-on（need → question）で持ち、書き込み時に検証する。自由属性 waiting-on は移行で辺に写す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-1cfb (D-84) 新しい gy の操作の集合は末端 20（書き 15、読み 5）、固有オプション 30 以下とし、AC-49 をこの数に改める | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-799e (D-85) publish の読み物は 6 つの問いに答える形で作り、マスターが読んで答えられなかった問いを後から足す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-d205 (D-86) 新しい gy の版は 0.5.0 とし、非互換の変更を一つの版にまとめる。1.0.0 は Kokopelli の実運用の移行と AC-51 / AC-52 の測定が済み publish の問いが安定した時点で改めて問う | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-648a (D-87) publish の既定は 1 ページの読み物（いま判断待ち、未決の論点、注意の件数、期間の変更）とし、ノードの記述（問い 1・3・4）は指定した ID の分だけ出す。全ノードの書き出しはしない | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-66ee (N-34) 要求の状態を 4 つにし、確定後は改訂・完了・中止の記録だけを持ち、ID を振り直して ref を持たせる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-3d8c (N-35) workflow.records / guards / スナップショット / lint 設定 / import 設定 / 申告オプション / gate を消し、gy.toml をスコープと出力先だけにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-0471 (N-36) bearer_count と need の状態を導出にし、handover を error と進行中だけにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-dbcf (N-37) 操作を末端 19 の閉じた集合にし、一つの意図を一つのコマンド・一つのトランザクションにする（lint は無い） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-da96 (N-39) 各コマンドの出力が、そのノードに無いものと次の操作を返し、同梱 skill を流れの説明だけにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-f312 (N-62) CLI (2a): decide / link / edit / undo の配線と、書きの入口での actor 検査の前倒し | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-2f21 (N-40) publish (a): views/publish の骨格と「いま判断待ち」（問い 2）、「未決の論点」（問い 6）、「注意」の節 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-bc0c (N-42) 新しい core の骨格を 4 層と型のモデルで作り、行数・glob・文字列キー・差分の大きさを測る道具を用意する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-271d (N-43) 文書と同梱 skill を新しい gy に合わせて書き直す（README 日英、architecture、CHEATSHEET、skill） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-3f44 (N-44) render と HTML 投影（html.rs、ui/、e2e/、テスト、同梱物、CI）を最初に消し、しがらみの無い木で始める | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-348d (N-45) 物理設計の基準を機械で測る道具 scripts/measure.sh を用意する（行数・関数の長さ・glob・文字列キー・テストの行数・差分・層の向き） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-a603 (N-46) 新しい core の model 層: 5 種のノードの型と不変条件を src/model/ のディレクトリ分割で作る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-edce (N-47) 辺の種類の組（論点 closes 決定、ニーズ targets 受け入れ条件など）を model の不変条件として検査し、外れた組の Link を作れなくする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-0cb7 (N-48) 正本の置き場所（XDG のデータディレクトリ + リポジトリのハッシュ）と形式の版 format の読み書き・判定・移行の入口の枠 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-bd8d (N-49) undo（逆変更の行の追記）と snapshot.json（commit ごとの原子的な書き直し、無くてもログの再生で復元） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-d1bf (N-50) 操作の共通の枠: gy.toml（スコープと出力先だけ）の読み込み、型付きノードの読み書きと ID・別名の解決を担う Repository、1 意図 1 トランザクションの実行の型、出力の型（ID・変えた項目・無いもの・次の操作） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-d514 (N-51) 操作 (a1): 辺を from 側だけに置き Need.targets の重複を消し、need add / need close を作る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-81d8 (N-52) 操作 (b): decide（ADR 有り無し、closes / narrows / supersedes / completes / widens と mark）と link（--mark、--remove） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-d1c6 (N-53) 操作 (c1): req add（--need 複数、--relies-on、--targets、--ref。同じ ref の未完了があれば拒否）と、resolve の ref 対応 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-447c (N-54) 操作 (d): edit（--title / --body / --set / --append、--reason 必須。状態と辺は変えない）と undo の操作 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-816a (N-55) 操作 (a2): question add / question close / criterion add / criterion satisfy の 4 操作（N-51 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-c980 (N-56) 操作 (c2): req approve（--design、--heard-by、--evidence）/ revise / done / cancel と、確定後の 3 つの記録（N-53 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-7cc1 (N-57) 読み (1a): views の骨格と show（複数 ID、ref でも引ける、種類ごとの逐語、--full、--json） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-58b4 (N-58) 読み (1b): list（--type / --status / --targets / --grep / --actor / --since。actor と since は書き込み単位）を views 層に作る（N-57 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-8996 (N-59) 読み (2a): HistoryEntry の seq、need の状態の導出、next を views 層に作る | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-54fe (N-61) CLI (1a): 新しい gy の CLI クレート gy5 の骨格（clap、GY_ACTOR、--json、-C、終了コード、正本の自動作成）と読み 4 の配線 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-f312 (N-62) CLI (2a): decide / link / edit / undo の配線と、書きの入口での actor 検査の前倒し | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-f65b (N-63) CLI (1b): need / question / criterion の 6 操作の配線とテスト（N-61 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-478b (N-64) CLI (2b): req add / approve / revise / done / cancel の配線と末端の数の測定（AC-49）（N-62 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-5091 (N-71) publish (b): ノードの節（問い 1・3・4）と「この期間の変更」（問い 5）、gy5 の配線、publish-sample.md（N-40 から分割） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-b6f3 (N-72) publish を 1 ページの読み物に直す: 既定はいま判断待ち・未決・注意の件数・期間の変更だけ、ノードの記述は publish <ID>... で指定した分だけ | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-c7b2 (Q-39) 新しい gy の正本の保存形式は何か（N-38 の前提。D-61 の要件から導く） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-875f (Q-40) 末端と固有オプションの数: undo と publish を含めると末端 20、固有オプションは distinct 30 前後になる。AC-49「末端 19 以下、固有オプション 20 前後」をどう扱うか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-6c8e (Q-41) ニーズの waiting-on（待っている論点）は自由属性の文字列のままでよいか、閉じた関係（need waits-on question）にして書き込み時に検証するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-037e (Q-42) publish の読み物が答えるマスターの問いは何か（roadmap 未決 2、第 3 段の入力。問いが揃うまで作らない） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-c5fc (Q-43) 新しい gy の版番号は 0.5.0 か 1.0.0 か（roadmap 未決 3） | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 2 | 2026-09-15 04:49 | lead | n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示） | updated | edit n-147b | 自分の台帳の切り替え (b) を完了した記録 |
| 3 | 2026-09-15 04:52 | lead | ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる | updated | criterion satisfy ac-8787 | 2026-09-15 マスターが .local/publish-sample.md（既定の出力 10 行）と .local/publish-sample-d78.md（publish D-78、8 行）を読み「読める長さの怪文書は初めて見た」。D-87 の形（既定は 1 ページ、ノードの記述は ID 指定の分だけ）で 6 つの問い（D-85）に gy を操作せずに答えられる。以前の 1,758 行の全ノードの書き出しはマスターが「人質でも取られない限り読まない」と却下し、D-87 で訂正した。Kokopelli の写しでは既定が 134 行（マスター待ちの論点 28 件） |
| 4 | 2026-09-15 04:55 | lead | ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる | updated | criterion satisfy ac-8787 | 2026-09-15 取り消し。マスター「何も読み取れませんが長さだけでよかったんですか」。長さは入口で、中身がまだ問いに答えていない。lead の早合点。読み物の中身を作り直してから改めて判定する |
| 5 | 2026-09-15 04:58 | lead | n-6eae publish <ID> を見出しと箇条書きの形にし、既定の読み物の見本を 2 つ（判断待ちが無いとき / あるとき）作ってマスターに見せる | created | need add n-6eae | need add |
| 6 | 2026-09-15 04:58 | lead | d-38b5 publish <ID> の答えは「見出しと箇条書き」の形にする: 見出しは旧 ID と短い題、節は 決めたこと / 理由 / 生んだ作業（状態付き）/ 状態（有効か、置き換えの有無）。要求は 来歴 / 状態 / 記録、ニーズは 目的 / 受け入れ条件 / 起票した要求 | created | decide d-38b5 | decide |
| 7 | 2026-09-15 04:58 | lead | n-6eae publish <ID> を見出しと箇条書きの形にし、既定の読み物の見本を 2 つ（判断待ちが無いとき / あるとき）作ってマスターに見せる | updated | link n-6eae spawned-by d-38b5 | link |
| 8 | 2026-09-15 05:05 | lead | ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている | updated | criterion satisfy ac-e8b7 | 2026-09-15 lead。N-44 から N-70 までの 40 ニーズすべてで、検収前に scripts/measure.sh の値（ファイル 300、関数 40、glob 0、文字列キー 1、テスト 300、score 600、層の向き）を測り、各ニーズの acceptance に残した。0.5.0 の時点: gy-ledger は違反 0（ファイル最大 293、文字列キーは model/node.rs の 1 か所）、gy（CLI）は違反 0（最大 275）、gy-migrate は文字列キーが legacy/ の 3 ファイルに限られ他 0（0.4 の辞書を読む役目のため対象外と依頼書で定めた）。分割で移動した行の再計上（N-68、N-70）は記録に残し対象外とした。差し戻しは 8 件、いずれも中身を見る前の測定か契約の読み合わせで見つけた |
| 9 | 2026-09-15 05:05 | lead | n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示） | updated | edit n-147b | 0.5.0 の準備が完了した記録 |
| 10 | 2026-09-15 05:13 | lead | n-6eae publish <ID> を見出しと箇条書きの形にし、既定の読み物の見本を 2 つ（判断待ちが無いとき / あるとき）作ってマスターに見せる | updated | need close n-6eae | {'code': 2, 'message': 'the mark "publish の既定は 1 ページの読み物" is not in the older decision\'s body or applicability conditions'} により取り下げ。publish に人間向けの形は足さない |
| 11 | 2026-09-15 05:13 | lead | n-02b9 WebUI（第 3 段）: マスターが「これは何で何と関係があるか」「今マスター待ちは何か」を把握できる画面。問いは D-85 の 6 つを入力にし、作るのはマスターの指示があってから | created | need add n-02b9 | need add |
| 12 | 2026-09-15 05:13 | lead | n-8f3f publish を記録の公開物にする: 指定した時点と範囲のノード・辺・履歴と診断結果を 1 ファイルに出し、--out（gy.toml の output）へ書いてコミットできる形にする。1 ページ化（N-72）は戻す | created | need add n-8f3f | need add |
| 13 | 2026-09-15 05:13 | lead | d-edb0 publish は開発の成果物としてコミットし後から過去の判断と経緯を振り返るための記録であり、開発中にマスターが読むものではない。「D-78 って何だっけ」に答えるのは WebUI のニーズで、gy のコマンドには人間向けの出力の形を足さない | created | decide d-edb0 | decide |
| 14 | 2026-09-15 05:13 | lead | n-02b9 WebUI（第 3 段）: マスターが「これは何で何と関係があるか」「今マスター待ちは何か」を把握できる画面。問いは D-85 の 6 つを入力にし、作るのはマスターの指示があってから | updated | link n-02b9 spawned-by d-edb0 | link |
| 15 | 2026-09-15 05:13 | lead | n-8f3f publish を記録の公開物にする: 指定した時点と範囲のノード・辺・履歴と診断結果を 1 ファイルに出し、--out（gy.toml の output）へ書いてコミットできる形にする。1 ページ化（N-72）は戻す | updated | link n-8f3f spawned-by d-edb0 | link |
| 16 | 2026-09-15 05:16 | lead | ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる | updated | edit ac-8787 | マスターの指摘（d-edb0 と 2026-09-15 の「完結した文書である必要がある」）で条件を言い換え |
| 17 | 2026-09-15 05:16 | lead | d-edb0 publish は開発の成果物としてコミットし後から過去の判断と経緯を振り返るための記録であり、開発中にマスターが読むものではない。「D-78 って何だっけ」に答えるのは WebUI のニーズで、gy のコマンドには人間向けの出力の形を足さない | updated | edit d-edb0 | マスターの追加の指摘を記録 |
| 18 | 2026-09-15 05:24 | lead | n-8f3f publish を記録の公開物にする: 指定した時点と範囲のノード・辺・履歴と診断結果を 1 ファイルに出し、--out（gy.toml の output）へ書いてコミットできる形にする。1 ページ化（N-72）は戻す | updated | need close n-8f3f | 2026-09-15 lead が検収。コードの measure 違反 0（score 522。公開物の行数は対象外）、テスト 190 件 0 failed。docs/publication/17.md（4,586 行）を原本を見ずに読み通し、読み方の節（30 行）で以降が読め、参照はすべて ID + 題名、決定は成立範囲と本文の全文と両向きの辺、履歴は表、診断に解き方の 1 文。コミット fb3ebf1 |
| 19 | 2026-09-15 05:24 | lead | ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる | updated | criterion satisfy ac-8787 | 2026-09-15 lead。gy -C docs/ledger publish --out docs/publication/{seq}.md で docs/publication/17.md（4,586 行）を生成しコミット fb3ebf1。原本を見ずに読み通せる完結した文書: 見出し（生成日時、seq 17、範囲、書き手、正本の場所）、読み方（5 種、4 状態、12 の関係、閉じ方、ID と別名、履歴の項目）、記録（258 ノードを scope・種類・created・ID 順で逐語、辺は両向きで相手の題名付き）、履歴（seq 1〜17 の書き込み単位を表で、ノードは ID + 題名）、診断（errors 0、warnings 1 件に解き方）。ID 単独の行 0 件（テストで保証）。マスターの指摘（振り返りの記録、原本を読まずに理解できる）に沿う |
| 20 | 2026-09-15 05:43 | lead | n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示） | updated | edit n-147b | 0.5.0 の公開の記録 |
| 21 | 2026-09-15 05:43 | lead | n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示） | updated | need close n-147b | 2026-09-15 公開済み。crates.io の gy-ledger 0.5.0 と gy 0.5.0、タグ v0.5.0（c2b9b97） |
| 22 | 2026-09-15 05:52 | lead | d-63f8 waits-on の先には論点に加えて要求も許し、要求が Done か Cancelled になれば待ちが解ける | created | decide d-63f8 | decide |
| 23 | 2026-09-15 05:52 | lead | n-b6a7 waits-on の先に要求を許す: model の組、derive の準備判定、gy-migrate の waiting-on の写し、テスト | created | need add n-b6a7 | need add |
| 24 | 2026-09-15 05:52 | lead | n-b6a7 waits-on の先に要求を許す: model の組、derive の準備判定、gy-migrate の waiting-on の写し、テスト | updated | link n-b6a7 spawned-by d-63f8 | link |
| 25 | 2026-09-15 06:01 | lead | n-b6a7 waits-on の先に要求を許す: model の組、derive の準備判定、gy-migrate の waiting-on の写し、テスト | updated | need close n-b6a7 | 2026-09-15 lead が検収。measure gy-ledger 違反 0（score 140）、gy-migrate は legacy/ の文字列キーのみ。テスト 191 件 0 failed。Kokopelli の写し（74fa00f、356 ノード）で waits-on 17（要求待ち 1 + belongs-to 10 + 元の 6）、skipped 0、next が 0.4 と同じ 6 件（N-1 / N-4 / N-11 / N-15 / N-23 / N-25）。コミット b86c2e9 |
| 26 | 2026-09-15 06:10 | lead | n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に） | updated | edit n-4fe0 | Kokopelli の実運用の移行を完了した記録 |
| 27 | 2026-09-15 06:12 | lead | n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に） | updated | edit n-4fe0 | Kokopelli の ref の大文字小文字の指摘で再移行した記録 |
| 28 | 2026-09-15 06:17 | lead | n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示） | updated | edit n-147b | 0.5.1 の公開の記録 |
| 29 | 2026-09-15 06:25 | lead | n-35cf show / list / publish のニーズの状態を next と同じ導出（open / closed / done）にする。list --status done を受ける | created | need add n-35cf | need add |
| 30 | 2026-09-15 06:28 | lead | n-9198 gy-migrate が既知の名前以外の自由属性を落とし報告にも出さない欠陥を直す: すべての残った属性を自由属性（配列は改行区切り）として写し、報告に一覧を出す。edit --append の意味（文字列に 1 行足す）を文書に明記 | created | need add n-9198 | need add |
| 31 | 2026-09-15 06:28 | lead | n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に） | updated | edit n-4fe0 | Kokopelli の実運用で見つかった移行の欠陥の記録 |
| 32 | 2026-09-15 06:29 | lead | n-35cf show / list / publish のニーズの状態を next と同じ導出（open / closed / done）にする。list --status done を受ける | updated | need close n-35cf | 2026-09-15 lead が検収。measure 違反 0（score 100）、テスト 0 failed。Kokopelli の台帳（読むだけ）で show N-25 が state: done、list --type need --status done に N-24 と N-25。コミット 1714824 |
| 33 | 2026-09-15 06:39 | lead | n-9198 gy-migrate が既知の名前以外の自由属性を落とし報告にも出さない欠陥を直す: すべての残った属性を自由属性（配列は改行区切り）として写し、報告に一覧を出す。edit --append の意味（文字列に 1 行足す）を文書に明記 | updated | need close n-9198 | 2026-09-15 lead が検収。measure は legacy/ の文字列キーのみ、テスト 0 failed。Kokopelli の写しで自由属性 27 名 255 値が写り、8 種 13 件を含む。next 6 件、辺 540、waits-on 17。コミット 6446450。ローカルにインストール済み |
| 34 | 2026-09-15 07:00 | lead | n-f088 ID の解決で、0 埋めの同一視を旧 ID の別名にだけ当て、gy が振ったハッシュ ID には当てない（d-0008 が D-8 と衝突する欠陥）。新しい ID は数字だけのハッシュを避ける | created | need add n-f088 | need add |
| 35 | 2026-09-15 07:06 | lead | n-f088 ID の解決で、0 埋めの同一視を旧 ID の別名にだけ当て、gy が振ったハッシュ ID には当てない（d-0008 が D-8 と衝突する欠陥）。新しい ID は数字だけのハッシュを避ける | updated | need close n-f088 | 2026-09-15 lead が検収。measure 違反 0（score 83）、テスト 0 failed。Kokopelli の台帳（読むだけ）で show D-8 が d-43e5、show d-0008 が d-0008、show D-08 が d-43e5。コミット 96dcfd3、ローカルにインストール |
| 36 | 2026-09-15 07:42 | lead | d-7c64 publish の出力は 1 ファイルでなく、スコープごとのディレクトリに 1 ノード 1 ファイルと索引 1 ファイルを書き出す形にし、出力先のスコープのディレクトリの中だけを作り直す | created | decide d-7c64 | decide |
| 37 | 2026-09-15 07:42 | lead | n-45ef publish (a): スコープごとのディレクトリ出力と 1 ノード 1 ファイル（関係と mark を相手の題名付きで。対象スコープのディレクトリだけを作り直す） | created | need add n-45ef | need add |
| 38 | 2026-09-15 07:42 | lead | n-45ef publish (a): スコープごとのディレクトリ出力と 1 ノード 1 ファイル（関係と mark を相手の題名付きで。対象スコープのディレクトリだけを作り直す） | updated | link n-45ef spawned-by d-7c64 | link |
| 39 | 2026-09-15 07:42 | lead | n-1aad edit --set scope=<名前> でノードを別のスコープへ移せるようにする（名前は gy.toml にあるものだけ。ID と辺は変わらない） | created | need add n-1aad | need add |
| 40 | 2026-09-15 07:42 | lead | n-1aad edit --set scope=<名前> でノードを別のスコープへ移せるようにする（名前は gy.toml にあるものだけ。ID と辺は変わらない） | updated | link n-1aad spawned-by d-7c64 | link |
| 41 | 2026-09-15 07:44 | lead | n-5c59 publish (b): スコープごとの索引 README.md（見出し・読み方・一覧とリンク・履歴・診断）と --since、gy.toml の output、文書、gy 自身の docs/publication の作り直し（n-45ef から分割） | created | need add n-5c59 | need add |
| 42 | 2026-09-15 07:44 | lead | n-5c59 publish (b): スコープごとの索引 README.md（見出し・読み方・一覧とリンク・履歴・診断）と --since、gy.toml の output、文書、gy 自身の docs/publication の作り直し（n-45ef から分割） | updated | link n-5c59 spawned-by d-7c64 | link |
| 43 | 2026-09-15 07:44 | lead | n-45ef publish (a): スコープごとのディレクトリ出力と 1 ノード 1 ファイル（関係と mark を相手の題名付きで。対象スコープのディレクトリだけを作り直す） | updated | edit n-45ef | 見込み 650〜750 行のため分割 |
| 44 | 2026-09-15 07:49 | lead | n-45ef publish (a): スコープごとのディレクトリ出力と 1 ノード 1 ファイル（関係と mark を相手の題名付きで。対象スコープのディレクトリだけを作り直す） | updated | need close n-45ef | 2026-09-15 lead が検収。measure 違反 0（score 368）、テスト 196 件 0 failed。gy 自身の台帳で --scope gy05 を一時ディレクトリへ出し、gy05/ の下に 4 種のディレクトリと 1 ノード 1 ファイル、決定は「## 関係」の節、出力先の無関係なファイルが残ることを確認。コミット d0b22ba |
| 45 | 2026-09-15 07:54 | lead | d-dcbb 0.4 の運用を決めた古い決定は消さず、新しい決定で置き換える（supersedes と mark）。公開物にも「置き換えられた」として残す。正本からノードを消す操作は提供しない | created | decide d-dcbb | decide |
| 46 | 2026-09-15 08:01 | lead | n-5c59 publish (b): スコープごとの索引 README.md（見出し・読み方・一覧とリンク・履歴・診断）と --since、gy.toml の output、文書、gy 自身の docs/publication の作り直し（n-45ef から分割） | updated | need close n-5c59 | 2026-09-15 lead が検収。measure 違反 0（score 405、生成物を除く）、テスト 198 件 0 failed。gy 自身の公開物 8 スコープ 280 ファイル、gy05 の索引 269 行（読み方・一覧・履歴・診断）。コミット 8d304b1、ローカルにインストール |
| 47 | 2026-09-15 08:03 | lead | n-ef16 publish のノードのファイルの形を整える: 関係は 1 か所（両向き）、成立範囲は 1 回、節の間に空行、書き手が無ければ行を出さない | created | need add n-ef16 | need add |
| 48 | 2026-09-15 08:03 | lead | n-ef16 publish のノードのファイルの形を整える: 関係は 1 か所（両向き）、成立範囲は 1 回、節の間に空行、書き手が無ければ行を出さない | updated | link n-ef16 spawned-by d-7c64 | link |
| 49 | 2026-09-15 08:08 | lead | n-79fb edit で、未記録の成立範囲だけを 1 回記録できる（--set decision_scope=<文>。記録済みは拒む）。--set key= は自由属性を消す | created | need add n-79fb | need add |
| 50 | 2026-09-15 08:08 | lead | n-79fb edit で、未記録の成立範囲だけを 1 回記録できる（--set decision_scope=<文>。記録済みは拒む）。--set key= は自由属性を消す | updated | link n-79fb spawned-by d-dcbb | link |
| 51 | 2026-09-15 08:08 | lead | n-1aad edit --set scope=<名前> でノードを別のスコープへ移せるようにする（名前は gy.toml にあるものだけ。ID と辺は変わらない） | updated | need close n-1aad | 2026-09-15 lead が検収。measure 違反 0（score 139）、テスト 0 failed。一時台帳で scope a → b の移動と、無い名前の Err を確認。コミット c67be17、ローカルにインストール |
| 52 | 2026-09-15 08:14 | lead | n-ef16 publish のノードのファイルの形を整える: 関係は 1 か所（両向き）、成立範囲は 1 回、節の間に空行、書き手が無ければ行を出さない | updated | need close n-ef16 | 2026-09-15 lead が検収。measure 違反 0（score 206）、テスト 0 failed。Kokopelli の台帳（読むだけ）の D-171 のファイルで関係の節 1 つ、成立範囲 1 回、節の前後に空行、索引に書き手の行なし。コミット 9273cc9、ローカルにインストール |
| 53 | 2026-09-15 08:18 | lead | n-79fb edit で、未記録の成立範囲だけを 1 回記録できる（--set decision_scope=<文>。記録済みは拒む）。--set key= は自由属性を消す | updated | need close n-79fb | 2026-09-15 lead が検収。measure 違反 0（score 201）、テスト 0 failed。一時台帳で記録済みの成立範囲の Err と、--set foo= で属性が消えることを確認。コミット 352b18c、ローカルにインストール |
| 54 | 2026-09-15 09:03 | lead | n-ff2b scope rename (a): 形式の版 2 と写し、ログの変更の種類 scope-renamed と再生 | created | need add n-ff2b | need add |
| 55 | 2026-09-15 09:03 | lead | ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける | updated | edit ac-09b1 | {'code': 2, 'message': 'the mark "末端 20（書き 15、読み 5）" is not in the older decision\'s body or applicability conditions'} で scope rename を足した |
| 56 | 2026-09-15 09:04 | lead | d-6e70 スコープ名の変更を操作 scope rename として閉じた集合に足す。履歴には「スコープ名の変更 旧 → 新（n ノード）」の 1 件の変更として残し、gy が gy.toml の [scopes.旧] を [scopes.新] に書き換える | created | decide d-6e70 | decide |
| 57 | 2026-09-15 09:04 | lead | n-ff2b scope rename (a): 形式の版 2 と写し、ログの変更の種類 scope-renamed と再生 | updated | link n-ff2b spawned-by d-6e70 | link |
| 58 | 2026-09-15 09:04 | lead | n-3f84 scope rename (b): 操作 scope_rename（Repository 経由の適用、履歴 1 行、undo、Outcome）とテスト（n-ff2b から分割） | created | need add n-3f84 | need add |
| 59 | 2026-09-15 09:04 | lead | n-24dd scope rename (c): CLI と gy.toml の書き換え（コメント・順序を保ち、失敗時はログを戻す）、文書、gy 自身の写しでの確認（n-ff2b から分割） | created | need add n-24dd | need add |
| 60 | 2026-09-15 09:04 | lead | n-3f84 scope rename (b): 操作 scope_rename（Repository 経由の適用、履歴 1 行、undo、Outcome）とテスト（n-ff2b から分割） | updated | link n-3f84 spawned-by d-6e70 | link |
| 61 | 2026-09-15 09:04 | lead | n-24dd scope rename (c): CLI と gy.toml の書き換え（コメント・順序を保ち、失敗時はログを戻す）、文書、gy 自身の写しでの確認（n-ff2b から分割） | updated | link n-24dd spawned-by d-6e70 | link |
| 62 | 2026-09-15 09:04 | lead | n-ff2b scope rename (a): 形式の版 2 と写し、ログの変更の種類 scope-renamed と再生 | updated | edit n-ff2b | 見込み 750〜900 行のため 3 分割 |
| 63 | 2026-09-15 09:13 | lead | n-ff2b scope rename (a): 形式の版 2 と写し、ログの変更の種類 scope-renamed と再生 | updated | need close n-ff2b | 2026-09-15 lead が検収。measure 違反 0（score 366）、テスト 210 件 0 failed。gy 自身の正本の写し（版 1）を新しいバイナリで開くと format.1.bak と events.jsonl.1.bak を残して版 2 になり、handover と next が同じ値。コミット 9f04bbc |
| 64 | 2026-09-15 09:18 | lead | n-3f84 scope rename (b): 操作 scope_rename（Repository 経由の適用、履歴 1 行、undo、Outcome）とテスト（n-ff2b から分割） | updated | need close n-3f84 | 2026-09-15 lead が検収。measure 違反 0（score 205）、テスト 0 failed。コミット a3fe323 |
| 65 | 2026-09-15 09:25 | lead | n-24dd scope rename (c): CLI と gy.toml の書き換え（コメント・順序を保ち、失敗時はログを戻す）、文書、gy 自身の写しでの確認（n-ff2b から分割） | updated | need close n-24dd | 2026-09-15 lead が検収。measure 違反 0（score 140）、テスト 0 failed。一時台帳で scope rename a z がノードを移し gy.toml のコメントと output を保って [scopes.z] に書き換え、履歴が 1 行、同名は Err。末端 21。コミット 53f9fb7、ローカルにインストール |
| 66 | 2026-09-15 09:25 | lead | ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける | updated | criterion satisfy ac-09b1 | 2026-09-15 d-6e70 で scope rename を足し末端 21 に改めたため一旦取り消し |
| 67 | 2026-09-15 09:25 | lead | ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける | updated | criterion satisfy ac-09b1 | 2026-09-15 lead。gy --help の末端は 21（書き 16: need add / close、question add / close、decide、req add / approve / revise / done / cancel、criterion add / satisfy、link、edit、undo、scope rename。読み 5）、固有オプション 30 以下。d-6e70 で改めた数のとおり |
| 68 | 2026-09-15 09:55 | lead | n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示） | updated | edit n-147b | 0.6.0 の公開の記録 |
| 69 | 2026-09-15 09:58 | lead | ac-a8ff (AC-51) Kokopelli の TEAM_AGENTS.md の gy に関する行数が、gy の出力が次の操作と不足を返すことで減る | updated | criterion satisfy ac-a8ff | 2026-09-15 lead が測定。Kokopelli の TEAM_AGENTS.md の gy に関する行（grep -ci gy）は移行前（bdedc1080）22 行、gy 0.5 の運用に書き直した後（1006ec8fd）20 行。全体は 175 → 172 行。減り幅は小さいが、records / guards / lint / cheatsheet の手順は消え、事実を得た担当が GY_ACTOR 付きで書き進行管理が list --since で照合する運用になった。Kokopelli の進行管理担当の完了報告（後始末: 決定 177 件の本文整理、未記録の成立範囲 81 件と空の AC 11 件の記入、scope rename、gy.toml をルートへ、docs/adr の削除、docs/gy-published への publish） |
| 70 | 2026-09-15 10:21 | lead | d-736e 0.4 系のスコープに残る未着手ニーズは、消した機能に伴うものと新しい gy が別の形で満たしたものに分けて閉じる。WebUI のニーズはマスターの指示があるまで立てない | created | decide d-736e | decide |
| 82 | 2026-09-15 10:21 | lead | n-02b9 WebUI（第 3 段）: マスターが「これは何で何と関係があるか」「今マスター待ちは何か」を把握できる画面。問いは D-85 の 6 つを入力にし、作るのはマスターの指示があってから | updated | need close n-02b9 | マスターの指示があってから立てる（d-edb0、d-736e）（d-736e） |
| 84 | 2026-09-15 10:22 | lead | ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける | updated | edit ac-09b1 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 98 | 2026-09-15 10:22 | lead | ac-5216 (AC-41) 確定後の転記が 0。進行管理が外の文書から gy へ写す記録が無い（今日は #6027 で約 22,800 bytes） | updated | edit ac-5216 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 103 | 2026-09-15 10:22 | lead | ac-6251 (AC-50) 利用者の文書と運用に採番の規則が現れない。ID の衝突と振り直しが起きない | updated | edit ac-6251 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 108 | 2026-09-15 10:22 | lead | ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す | updated | edit ac-7652 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 109 | 2026-09-15 10:22 | lead | ac-7670 (AC-45) 1 コマンドの書き込みは全部書けるか全部書かないか。途中失敗で中途半端な状態が残らない | updated | edit ac-7670 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 112 | 2026-09-15 10:22 | lead | ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる | updated | edit ac-8787 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 113 | 2026-09-15 10:22 | lead | ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ | updated | edit ac-8ca4 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 116 | 2026-09-15 10:22 | lead | ac-9be9 (AC-40) gy.toml に書けるのはスコープと出力先だけで、他のキーは読み込み時にエラー。Kokopelli の gy.toml 2,117 行が 16 行相当で読み込める | updated | edit ac-9be9 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 119 | 2026-09-15 10:22 | lead | ac-a8ff (AC-51) Kokopelli の TEAM_AGENTS.md の gy に関する行数が、gy の出力が次の操作と不足を返すことで減る | updated | edit ac-a8ff | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 123 | 2026-09-15 10:22 | lead | ac-b0f3 (AC-46) すべての書き込みに、いつ・誰が（GY_ACTOR 必須）・なぜ・出典が残り、undo できる | updated | edit ac-b0f3 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 126 | 2026-09-15 10:22 | lead | ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ | updated | edit ac-c21e | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 127 | 2026-09-15 10:22 | lead | ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている | updated | edit ac-c2bd | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 128 | 2026-09-15 10:22 | lead | ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない | updated | edit ac-ce32 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 131 | 2026-09-15 10:22 | lead | ac-d2ab (AC-57) render サブコマンド、html.rs、ui/、e2e/、html_render のテスト、同梱の dist と template、CI の該当ジョブが無く、cargo test --workspace と cargo package --list にそれらが現れない | updated | edit ac-d2ab | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 132 | 2026-09-15 10:22 | lead | ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0 | updated | edit ac-d8f9 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 134 | 2026-09-15 10:22 | lead | ac-d95a (AC-56) README 日英・docs・CHEATSHEET・同梱 skill が新しい gy の 20 の操作（書き 15、読み 5。D-84）と一つの進め方だけを説明し、消した設定・操作・規則への言及が無い（grep で 0） | updated | edit ac-d95a | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 136 | 2026-09-15 10:22 | lead | ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている | updated | edit ac-e8b7 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 138 | 2026-09-15 10:22 | lead | ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%） | updated | edit ac-fc89 | 本文（測り方）が空の受け入れ条件を埋める（片付け） |
| 139 | 2026-09-15 10:22 | lead | ac-9be9 (AC-40) gy.toml に書けるのはスコープと出力先だけで、他のキーは読み込み時にエラー。Kokopelli の gy.toml 2,117 行が 16 行相当で読み込める | updated | edit ac-9be9 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 140 | 2026-09-15 10:22 | lead | ac-5216 (AC-41) 確定後の転記が 0。進行管理が外の文書から gy へ写す記録が無い（今日は #6027 で約 22,800 bytes） | updated | edit ac-5216 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 141 | 2026-09-15 10:22 | lead | ac-fc89 (AC-42) 要求ノードは確定前の記述と辺と ref だけ。スナップショットが無い（今日は 431 KB のうち 95%） | updated | edit ac-fc89 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 142 | 2026-09-15 10:22 | lead | ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ | updated | edit ac-c21e | gy05 の受け入れ条件の測り方を書く（片付け） |
| 143 | 2026-09-15 10:22 | lead | ac-ce32 (AC-44) ソースリポジトリの作業ツリーに gy のファイルが無く、台帳のための git も worktree も要らない | updated | edit ac-ce32 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 144 | 2026-09-15 10:22 | lead | ac-7670 (AC-45) 1 コマンドの書き込みは全部書けるか全部書かないか。途中失敗で中途半端な状態が残らない | updated | edit ac-7670 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 145 | 2026-09-15 10:22 | lead | ac-b0f3 (AC-46) すべての書き込みに、いつ・誰が（GY_ACTOR 必須）・なぜ・出典が残り、undo できる | updated | edit ac-b0f3 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 146 | 2026-09-15 10:22 | lead | ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0 | updated | edit ac-d8f9 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 147 | 2026-09-15 10:22 | lead | ac-8ca4 (AC-48) gy のコマンド・設定・ID・項目名に特定の外部サービスの語が無く、外への参照は ref 1 つ | updated | edit ac-8ca4 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 148 | 2026-09-15 10:22 | lead | ac-09b1 (AC-49) 末端のサブコマンドが 21 以下（書き 16、読み 5）、固有オプションが 30 以下（今 29 と 36）。今日の定型 9 つがそれぞれ 1 コマンドで書ける | updated | edit ac-09b1 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 149 | 2026-09-15 10:22 | lead | ac-6251 (AC-50) 利用者の文書と運用に採番の規則が現れない。ID の衝突と振り直しが起きない | updated | edit ac-6251 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 150 | 2026-09-15 10:22 | lead | ac-a8ff (AC-51) Kokopelli の TEAM_AGENTS.md の gy に関する行数が、gy の出力が次の操作と不足を返すことで減る | updated | edit ac-a8ff | gy05 の受け入れ条件の測り方を書く（片付け） |
| 151 | 2026-09-15 10:22 | lead | ac-8787 (AC-52) publish の出力が、指定した時点と範囲の記録（ノード・辺・履歴）と診断結果を含む完結した文書で、原本（正本）を読まずに理解でき、コミットして後から判断と経緯を振り返れる | updated | edit ac-8787 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 152 | 2026-09-15 10:22 | lead | ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す | updated | edit ac-7652 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 153 | 2026-09-15 10:22 | lead | ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている | updated | edit ac-e8b7 | gy05 の受け入れ条件の測り方を書く（片付け） |
| 154 | 2026-09-15 10:22 | lead | ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている | updated | edit ac-c2bd | gy05 の受け入れ条件の測り方を書く（片付け） |
| 155 | 2026-09-15 10:22 | lead | ac-d95a (AC-56) README 日英・docs・CHEATSHEET・同梱 skill が新しい gy の 20 の操作（書き 15、読み 5。D-84）と一つの進め方だけを説明し、消した設定・操作・規則への言及が無い（grep で 0） | updated | edit ac-d95a | gy05 の受け入れ条件の測り方を書く（片付け） |
| 156 | 2026-09-15 10:22 | lead | ac-d2ab (AC-57) render サブコマンド、html.rs、ui/、e2e/、html_render のテスト、同梱の dist と template、CI の該当ジョブが無く、cargo test --workspace と cargo package --list にそれらが現れない | updated | edit ac-d2ab | gy05 の受け入れ条件の測り方を書く（片付け） |

## 診断

エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。警告は注意が要る状態で、`handover` に一覧が出る。

errors: 0
warnings: 0
