# n-1b72 (N-27) 押せる見た目と押せる実体を一致させ、Overview の数値カードを一覧への入口にする

- 種類: need
- scope: html_projection
- created: 2026-09-13
- 状態: closed
- 別名: N-27

## 関係

- depends-on n-b4d3 (N-25) 画面のコードを ui/ 配下の TypeScript モジュールに再構成し、bun で束ねた成果物を html.rs が埋め込む形にする
- spawned-by d-6e4d (D-44) 押せるものは押せる見た目を持ち、押せる見た目のものは押すと何かが起きる
- targets ac-9e06 (AC-32) クリックハンドラを持つ全要素が button/link の役割とポインタカーソルを持ち、Overview の数値カードはそれぞれの一覧へ至るか枠を持たず、ポインタカーソルを持つのに何も起きない要素が 0 件である

## 本文

## 要求（2026-09-14、マスター承認済み。開発担当 deck へ振り出し）

根拠は D-44、D-40、D-39。状態は N-22 の後（fc71408）。判定は AC-32。受領は kuroko の目視を含む。

### 満たす条件

1. **押せる実体には押せる見た目。** クリックハンドラを持つ全要素（一覧の行、グラフのノードとクラスタ、Overview のカード、チップ、タブ）は `button` か `a` の役割を持ち、ポインタカーソルと hover の視覚変化を持つ。一覧の行は行全体がリンクとして見える（hover で行が変わる）。
2. **押せる見た目には押せる実体。** ポインタカーソルや枠付きカードの見た目なのに何も起きない要素を 0 にする。Overview の数値カード（criteria satisfied / criteria / questions / waiting-on / needs offered by next / lint errors / warnings）は、その数が指す一覧へ至るリンクにする（例: needs offered by next → 種別 need かつ next が提示する need に絞った一覧、lint errors → Blockers タブ）。一覧に対応が無いものは枠を外して文字だけにする。Requirement states の行も、その状態に絞った一覧へ至る。
3. **切り替えは状態を自身に表示する。** 種別チップは選択中と非選択が一目で分かる（塗りと枠の差だけでなく、選択中にチェックか太字など第二の手掛かりを持つ）。All types は全選択の時だけ押し込まれた見た目。Lineage（genealogy）は on/off が自身に表示される。
4. **同じ操作は 1 か所。** 焦点移動のボタンは詳細パネルかグラフ操作の帯のどちらか一方に置く（グラフ操作の帯には Node ID 入力があるので、詳細パネル側を残しグラフ帯からは「選択中のノードを中心に」という形で 1 つに統合する、など）。
5. **検査可能にする。** E2E で、ポインタカーソルを持つ全要素にクリックハンドラか href があること、Overview の各カードをクリックして一覧の件数がカードの数と一致すること、チップの選択状態が aria-pressed で読めることを測る。

### 検証

- E2E: 上記 5 の検査。既存 36 件は通す。
- 目視: 1440px のスクリーンショット（Overview、一覧、チップの選択状態）を報告に含める。kuroko が現物で確認してから受領する。

### 共通の維持条件と完了

- 派生表示の契約、埋め込みエスケープ、単一 HTML・ネットワークなしは不変。互換な変更（z）。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`e2e` の `npm test`、`ui` の `bun run check` が exit 0。
- 受け入れ条件を evidence 付きで satisfy。need ごとにコミット、push と公開はマスターの指示待ち。報告は `herdr agent prompt kuroko` で D-33 の形、スクリーンショットの path を含める。判断に迷う点、原則（D-38〜D-45）と衝突する点は question として台帳に置いて私へ送る。

## 照合（kuroko、2026-09-14）

deck の報告をコミット 2d3d3ce で照合。隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed、bun run check exit 0、E2E 39 passed / 0 failed。gy 自身の台帳（136 ノード）を描いて terminal-browser で確認: ポインタカーソルを持つ 284 要素のうち button / a / role を持たないものは 0。「needs offered by next」カードは a 要素で、実マウス入力でクリックすると need のチップが ✓ で押され、一覧は 23 行（カードの数と一致）、状態行は「23 results · Graph: 23 shown · 0 hidden · 2 filters」、hash に overview=next が載る（D-47）。Lineage は「Lineage: off」と状態を自身に表示。

追補が要る点: 一覧の行は tr にクリックハンドラがあり hover で背景が変わるが、カーソルは auto で役割も無い（a は ID セルだけ）。AC-32 の「クリックハンドラを持つ全要素が button/link の役割とポインタカーソルを持つ」に届いていない。行全体に role=link（または各セルの内容を同じ href の a に）とポインタカーソルを付けたうえで受領する。

## 追補の照合（kuroko、2026-09-14）

322a8ba を隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed、bun check exit 0、E2E は 39 passed / **1 failed**（単独でも再現）。落ちたのは `actions.spec.mjs` の「pointer elements have native actions or registered role handlers」で、tr に付けた pointer が tr / td（170 要素）に及び、この検査は `closest('a[href],button,...')` で祖先の操作を探すため、子孫の a を実ヒット対象とする行を不合格にする。実ヒット対象（elementFromPoint）は 6/6 列で `a[data-record]` かつ pointer で、D-48 の契約自体は成立している。矛盾しているのは検査の方法（計算スタイル + 祖先探索）で、D-48 の「検査は実ヒット対象で行う」に揃える必要がある。deck の報告は「該当 E2E 1 件」のみで全件実行が無く、維持条件（`npm test` exit 0）を満たしていない。差し戻し。

## 受領（kuroko、2026-09-14）

cad46c1 を隔離 worktree で再実行: fmt exit 0、clippy 警告 0、Rust 71 passed、bun check exit 0。actions.spec は単独で 4/4 passed（前回の 170 件の不合格は解消）。全件は 39 passed / 1 failed だが、落ちたのは location.spec「URL selects IDs, reports missing IDs, and restores state through history」で、goBack 直後の hash の tab が jams のはずが overview になる。単独 3 回で 2 回成功 1 回失敗（時間依存）。N-27 の変更と無関係の既存テスト（N-19 由来）の不安定さなので N-27 は受領し、不安定さは N-28 として切り出す。

## 閉じ方

- 事実で閉じた（migrated: complete）

## 自由属性

- 無し

