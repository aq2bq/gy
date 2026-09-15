# 公開の手順

この文書は、gy の版の決め方、準備、公開の手順を定める。誰が何を判断するかは `AGENTS.md` にあり、ここには手順と確認項目を書く。

## 版を決める

作業ツリーと直前の公開版を、Rust の公開 API、CLI の表面、保存の形式、`gy.toml`、診断と終了コード、履歴の解釈のそれぞれで比べる。`0.y.z` では互換な変更が `z`、非互換な変更が `y` を動かす。`1.0.0` 以降は semver に従う。テストの量や差分の大きさでは決めない。非互換の変更は 1 つの版にまとめる。

開発中のビルドは、公開まで直前の版番号を保つ。公開のときに、`gy-ledger`・`gy`・`gy-migrate` の版と、CLI が依存するクレートの指定と `Cargo.lock` を 1 つのコミットで揃える。このコミットは機能のコミットと分ける。コミットメッセージは英語で、変更の理由を書く。

## 準備する（取り消せる）

1. 英語の `CHANGELOG.md` と、移行の案内（`docs/migration-0.5.md`）を書く。利用者に残る手作業は、CHANGELOG と README の両方に書く。同じコマンドを再実行して安全かどうかも明記する。
2. 移行の案内にあるコード例を、下流のクレートから実行して確かめる。読める例とコンパイルが通る例は別である。`#[non_exhaustive]` の型は、定義したクレートの外では構造体リテラルと `..Default::default()` を受け付けない。下流は `Default::default()` を作って公開フィールドへ代入する。
3. `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked` を実行する。
4. `cargo publish --workspace --dry-run --locked` を実行する。`gy-ledger` と `gy` を依存の向きの順に package して検証する（`publish = false` の `gy-migrate` と `gy-core` は対象外）。`--allow-dirty` はコミット前の確認にだけ使い、公開はきれいな検証済みのコミットから行う。

> 同じ版番号で dry-run を繰り返すと、前回の検証で組んだ gy-ledger の成果物（`target/` と `~/.cargo/registry/src/*/gy-ledger-<版>`）が再利用され、`gy` の検証が古い API で失敗することがある。その場合は `cargo clean` と当該ディレクトリの削除の後に dry-run をやり直す（2026-09-15 の 0.5.0 の準備で確認）。

5. クレートごとに `cargo package --list` を実行し、埋め込んだファイルが package に含まれることを確かめる。欠けたファイルは手元のビルドを通り、公開したクレートでだけ壊れる。
6. 版を揃えたコミットを行う。

## 公開する（取り消せない）

crates.io への公開は取り消せない。`yank` は新しい依存がその版を選ぶのを止めるだけで、番号は消費されたまま残る。取り消せる準備が終わり、公開の指示が出てから行う。

1. `cargo publish -p gy-ledger --locked`、次に `cargo publish -p gy --locked`。`gy-migrate` と `gy-core` は残る 0.4 の移行のためのもので、0.5 では公開しない（`publish = false`。移行が終わった版で消す）。
2. 公開のコミットと、対応する `vX.Y.Z` のタグを push する。
3. レジストリから入れて、バイナリを一度動かす: `cargo install gy --locked && gy --version`。
4. 公開した版と更新のコマンドを利用者へ伝える。`--locked` は同梱の依存の版を再現する。

公開済みの版は不変の成果物である。修正は次の適切な版で届ける。
