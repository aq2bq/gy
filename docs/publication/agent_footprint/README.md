# gy の公開物 — agent_footprint

- 生成: 2026-09-15T07:59:11Z
- seq: 45
- scope: agent_footprint
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
- [n-3ed7 (N-18) AGENTS.md を判断の所在と型だけの記述へ圧縮し、手順とツールの癖を各所へ移す](needs/n-3ed7-AGENTS-md-を判断の所在と型だけの記述へ圧縮し-手順とツールの癖を各所へ.md) — open
- [n-6dab (N-16) workflow.rs を関心ごとに分割し、テストを有効な基準台帳のフィクスチャから構成する](needs/n-6dab-workflow-rs-を関心ごとに分割し-テストを有効な基準台帳のフィクスチャ.md) — open
- [n-adaa (N-17) 変更系コマンドの返却を識別子と更新項目に絞り、本文は show で取得する形にする](needs/n-adaa-変更系コマンドの返却を識別子と更新項目に絞り-本文は-show-で取得する形にす.md) — open
- [n-9ed7 (N-30) E2E の 1 回の実行を軽くし、fixture の再生成をソースの変化がある時だけにする](needs/n-9ed7-E2E-の-1-回の実行を軽くし-fixture-の再生成をソースの変化がある時.md) — closed
### questions
- [q-3693 (Q-28) N-16 の API 維持対象を実在する crate 直下の公開パスとするか、workflow 配下の公開パスも追加するか](questions/q-3693-N-16-の-API-維持対象を実在する-crate-直下の公開パスとするか-w.md) — closed
- [q-c0fd (Q-29) N-17 の本文のみの更新とノードを持たない init の結果を、識別子・属性名だけの契約でどう表すか](questions/q-c0fd-N-17-の本文のみの更新とノードを持たない-init-の結果を-識別子-属性名.md) — closed
- [q-e7c8 (Q-27) 変更系コマンドの出力から本文を外す範囲をどう定めるか](questions/q-e7c8-変更系コマンドの出力から本文を外す範囲をどう定めるか.md) — closed
- [q-657a (Q-35) N-30 の直列性能検査を維持した実測が AC-35 の3分の1目標に届かない場合の完了範囲](questions/q-657a-N-30-の直列性能検査を維持した実測が-AC-35-の3分の1目標に届かない場.md) — closed
- [q-f7b4 (Q-36) 0.4.0 が未公開だった事実を踏まえ、次の公開版を 0.4.0（0.3.1 以降の全差分を 1 つの非互換版にまとめる）とするか 0.5.0（指示どおり、未公開の 0.4.0 を飛ばす）とするか](questions/q-f7b4-0-4-0-が未公開だった事実を踏まえ-次の公開版を-0-4-0-0-3-1-以.md) — closed
### decisions
- [d-6ba8 (D-33) 契約と証拠の正本は台帳であり、報告・連絡・完了報告は台帳の ID とそこからの差分・測定結果・残った判断で書く](decisions/d-6ba8-契約と証拠の正本は台帳であり-報告-連絡-完了報告は台帳の-ID-とそこからの差.md)
- [d-75c2 (D-35) N-16 の分割は gy_core 直下の既存の公開パスと型の形を維持し、workflow モジュールは非公開のまま内部で分割する](decisions/d-75c2-N-16-の分割は-gy_core-直下の既存の公開パスと型の形を維持し-wor.md)
- [d-922b (D-36) 変更系の結果はノード要約 id / type / scope / changed_attributes / body_changed とし、init は解決した root と scope を返す](decisions/d-922b-変更系の結果はノード要約-id-type-scope-changed_attri.md)
- [d-c956 (D-34) 変更系コマンドは id と更新した属性名だけを返し、本文と全属性は show / find が返す](decisions/d-c956-変更系コマンドは-id-と更新した属性名だけを返し-本文と全属性は-show-f.md)
- [d-ecff (D-32) AGENTS.md は判断の所在と型だけを記述し、手順・ツールの癖・過去の失敗はツール、docs、台帳の decision に置く](decisions/d-ecff-AGENTS-md-は判断の所在と型だけを記述し-手順-ツールの癖-過去の失敗は.md)
- [d-3c63 (D-55) README.md も README.ja.md と同じ節構成の英語の読み物として書き、日英は事実と節の集合で同期する](decisions/d-3c63-README-md-も-README-ja-md-と同じ節構成の英語の読み物とし.md)
- [d-441b (D-51) 要求と設計案が名指しする既存の仕組み（関数・公開パス・UI の操作）は、書く前に存在をソースで確かめる](decisions/d-441b-要求と設計案が名指しする既存の仕組み-関数-公開パス-UI-の操作-は-書く前に.md)
- [d-5d09 (D-53) 次の公開版は 0.4.0 とし、0.3.1 以降の全差分を 1 つの非互換版にまとめる](decisions/d-5d09-次の公開版は-0-4-0-とし-0-3-1-以降の全差分を-1-つの非互換版にま.md)
- [d-819b (D-54) README.ja.md は英語版の逐文訳ではなく、人間が通読する日本語の読み物として書く](decisions/d-819b-README-ja-md-は英語版の逐文訳ではなく-人間が通読する日本語の読み物.md)
- [d-f6c0 (D-52) ローカルの既定の E2E は機能 spec だけを走らせ、性能ゲートは CI と明示の入口（test:perf / test:all）で走らせる](decisions/d-f6c0-ローカルの既定の-E2E-は機能-spec-だけを走らせ-性能ゲートは-CI-と.md)
### requirements
- 無し
### criteria
- [ac-07ca (AC-19) gy-core/src/workflow.rs の記録比較と履歴検査が設定 schema と別ファイルにあり、分割の前後で cargo test --workspace --locked の結果が同一である](criteria/ac-07ca-gy-core-src-workflow-rs-の記録比較と履歴検査が設定-sc.md) — satisfied
- [ac-2101 (AC-21) 変更系コマンドの JSON 出力に入力として渡した本文が含まれず、CLI と MCP で同じ形を返す](criteria/ac-2101-変更系コマンドの-JSON-出力に入力として渡した本文が含まれず-CLI-と-M.md) — satisfied
- [ac-510f (AC-22) AGENTS.md が判断の所在と型だけを記述し、禁止形の規則文が0件で、マスター裁量の項目が改稿前と同一である](criteria/ac-510f-AGENTS-md-が判断の所在と型だけを記述し-禁止形の規則文が0件で-マスタ.md) — satisfied
- [ac-7b51 (AC-20) 統合テストが有効な基準台帳を作る共通フィクスチャから始まり、handover と下流の実行例がそのフィクスチャから構成される](criteria/ac-7b51-統合テストが有効な基準台帳を作る共通フィクスチャから始まり-handover-と.md) — satisfied
- [ac-f68f (AC-35) e2e のローカル既定（npm test、機能 spec）が fixture 再利用時に 30 秒台で完了し、npm run test:all は CI と同じ全件を実行して成否が 1 ワーカー実行と同一である](criteria/ac-f68f-e2e-のローカル既定-npm-test-機能-spec-が-fixture-再.md) — satisfied

## 履歴

| seq | 日時 | 書き手 | ノード | 何を | なぜ | 出典 |
|---|---|---|---|---|---|---|
| 1 | 2026-09-15 04:49 | lead | ac-07ca (AC-19) gy-core/src/workflow.rs の記録比較と履歴検査が設定 schema と別ファイルにあり、分割の前後で cargo test --workspace --locked の結果が同一である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-7b51 (AC-20) 統合テストが有効な基準台帳を作る共通フィクスチャから始まり、handover と下流の実行例がそのフィクスチャから構成される | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-2101 (AC-21) 変更系コマンドの JSON 出力に入力として渡した本文が含まれず、CLI と MCP で同じ形を返す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-510f (AC-22) AGENTS.md が判断の所在と型だけを記述し、禁止形の規則文が0件で、マスター裁量の項目が改稿前と同一である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | ac-f68f (AC-35) e2e のローカル既定（npm test、機能 spec）が fixture 再利用時に 30 秒台で完了し、npm run test:all は CI と同じ全件を実行して成否が 1 ワーカー実行と同一である | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-ecff (D-32) AGENTS.md は判断の所在と型だけを記述し、手順・ツールの癖・過去の失敗はツール、docs、台帳の decision に置く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-6ba8 (D-33) 契約と証拠の正本は台帳であり、報告・連絡・完了報告は台帳の ID とそこからの差分・測定結果・残った判断で書く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-c956 (D-34) 変更系コマンドは id と更新した属性名だけを返し、本文と全属性は show / find が返す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-75c2 (D-35) N-16 の分割は gy_core 直下の既存の公開パスと型の形を維持し、workflow モジュールは非公開のまま内部で分割する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-922b (D-36) 変更系の結果はノード要約 id / type / scope / changed_attributes / body_changed とし、init は解決した root と scope を返す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-441b (D-51) 要求と設計案が名指しする既存の仕組み（関数・公開パス・UI の操作）は、書く前に存在をソースで確かめる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-f6c0 (D-52) ローカルの既定の E2E は機能 spec だけを走らせ、性能ゲートは CI と明示の入口（test:perf / test:all）で走らせる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-5d09 (D-53) 次の公開版は 0.4.0 とし、0.3.1 以降の全差分を 1 つの非互換版にまとめる | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-819b (D-54) README.ja.md は英語版の逐文訳ではなく、人間が通読する日本語の読み物として書く | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | d-3c63 (D-55) README.md も README.ja.md と同じ節構成の英語の読み物として書き、日英は事実と節の集合で同期する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-6dab (N-16) workflow.rs を関心ごとに分割し、テストを有効な基準台帳のフィクスチャから構成する | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-adaa (N-17) 変更系コマンドの返却を識別子と更新項目に絞り、本文は show で取得する形にする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-3ed7 (N-18) AGENTS.md を判断の所在と型だけの記述へ圧縮し、手順とツールの癖を各所へ移す | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | n-9ed7 (N-30) E2E の 1 回の実行を軽くし、fixture の再生成をソースの変化がある時だけにする | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-e7c8 (Q-27) 変更系コマンドの出力から本文を外す範囲をどう定めるか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-3693 (Q-28) N-16 の API 維持対象を実在する crate 直下の公開パスとするか、workflow 配下の公開パスも追加するか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-c0fd (Q-29) N-17 の本文のみの更新とノードを持たない init の結果を、識別子・属性名だけの契約でどう表すか | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-657a (Q-35) N-30 の直列性能検査を維持した実測が AC-35 の3分の1目標に届かない場合の完了範囲 | created | migrate from 0.4 | publication/0.4 0d42e55 |
| 1 | 2026-09-15 04:49 | lead | q-f7b4 (Q-36) 0.4.0 が未公開だった事実を踏まえ、次の公開版を 0.4.0（0.3.1 以降の全差分を 1 つの非互換版にまとめる）とするか 0.5.0（指示どおり、未公開の 0.4.0 を飛ばす）とするか | created | migrate from 0.4 | publication/0.4 0d42e55 |

## 診断

エラーは正本の整合の破れ。`show <ID>` で該当ノードを読み、`edit` か `link` で直す。警告は注意が要る状態で、`handover` に一覧が出る。

errors: 0
warnings: 1
- criteria with an empty body: 4
