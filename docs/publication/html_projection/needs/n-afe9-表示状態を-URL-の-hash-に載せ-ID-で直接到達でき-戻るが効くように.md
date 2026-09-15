# n-afe9 (N-19) 表示状態を URL の hash に載せ、ID で直接到達でき、戻るが効くようにする

- 種類: need
- scope: html_projection
- created: 2026-09-13
- 状態: closed
- 別名: N-19

## 関係

- spawned-by d-626e (D-38) 場所は URL が持つ: 表示状態は hash に載り、ブラウザの戻るが効き、ID を貼れば開く
- targets ac-5b53 (AC-24) gy.html#<ID> を開くとそのノードの詳細が開いており、焦点・選択・絞り込み・検索語の変更が hash に反映され、ブラウザの戻るで直前の表示状態に戻る

## 本文

## 要求（2026-09-13、マスター承認済み。開発担当 deck へ振り出し）

根拠は D-37（業務: 指名到達・辿る）と D-38。状態は `main` beb4449。判定は AC-24。

### 満たす条件

1. `gy.html#<ID>` を開くと、そのノードが選択され詳細が開いている。ID だけの hash（`#Q-93`）を受け付ける。存在しない ID は画面に「見つからない」と出す。
2. 表示状態（選択ノード、焦点とその半径、種別・スコープ・状態の絞り込み、検索語、左パネルのタブ、genealogy の on/off）は hash に載り、hash から復元できる。同じ hash を別のタブで開けば同じ画面になる。
3. 状態を変える操作は履歴に 1 段を積み、ブラウザの戻る/進むで直前の表示状態に戻る。ズームとパンは履歴に積まない。
4. hash の形式は `state.js` に 1 か所で定義し、読み書きを同じ関数が担う。

### 共通の維持条件と完了

- 派生表示の契約（architecture.md HTML projection 節: lint 等の判断を画面で計算しない、埋め込みエスケープ、バナーの件数表示）と Rust 側のテストを維持する。HTML の変化は互換な変更（z）に属する。
- 既存の E2E（`e2e/tests/`）は通す。D-39 により期待値が変わるテスト（クリック=焦点移動を前提にしたもの）は、その理由を D-39 として記して変更する。
- 検証は `e2e/` の Playwright テストで行い、受け入れ条件の測定値を evidence に書く。terminal-browser は目視用。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`e2e` の `npm test` が exit 0。
- need ごとにコミットし、push と公開はマスターの指示を待つ。報告は `herdr agent prompt kuroko` で D-33 の形。判断に迷う点、原則（D-38〜D-43）と衝突する点は question として台帳に置いて私へ送る。

## 完了確認（kuroko、2026-09-13）

deck の報告をコミット ba302ab で照合。kuroko 側は deck の作業ツリーと分けた worktree（e79b617）で再実行: fmt exit 0、clippy 警告 0、cargo test --workspace --locked 71 passed / 0 failed、e2e npm test 23 passed / 0 failed。作業ツリー上で走らせた最初の E2E は N-20 作業中の未コミット変更を含んでいたため 3 件落ちたが、コミット済み HEAD では再現せず。受領。push・公開は未実施。

## 閉じ方

- 事実で閉じた（未実装のまま閉じた。HTML 投影は D-79 で消す (D-80)）

## 自由属性

- 無し

