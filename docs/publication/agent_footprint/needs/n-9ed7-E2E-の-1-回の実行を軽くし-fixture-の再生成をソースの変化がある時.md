# n-9ed7 (N-30) E2E の 1 回の実行を軽くし、fixture の再生成をソースの変化がある時だけにする

state: closed
scope: agent_footprint
created: 2026-09-14
  spawned-by d-c567 (D-46) CI に追加した検査は、同一コマンドのローカル成功と欠陥注入での失敗を根拠に受け入れ、GitHub 上の実行結果は push 後に追補の evidence として確かめる
  targets ac-f68f (AC-35) e2e のローカル既定（npm test、機能 spec）が fixture 再利用時に 30 秒台で完了し、npm run test:all は CI と同じ全件を実行して成否が 1 ワーカー実行と同一である
## 出所

2026-09-14、マスターの「実装に想定以上の時間がかかった」報告を kuroko が調査。need あたりの実作業は 5〜27 分だが、全件 E2E は 1 回約 95 秒（deck 実測 96.1 / 97.4 / 95.6 秒）で、毎回 `cargo build` と全 fixture の HTML 再生成を含む。1 need につき deck と kuroko で 2〜4 回走るので、5〜7 分が検証で消える。`playwright.config.mjs` は workers: 1、fullyParallel: false。CI も 1 ワーカー。

## 要求（マスター承認済み「すぐやってほしい」。開発担当 deck へ振り出し）

根拠は D-46（CI の検査が契約）と D-45。判定は AC-35。

### 満たす条件

1. **fixture の再利用。** `fixtures.mjs` は、CLI バイナリの内容ハッシュと `fixtures.mjs` 自身のハッシュを `.generated/stamp.json` に記録し、両方が一致すれば生成を省く。`cargo build` は毎回走ってよい（変更が無ければ数秒）。`--force` で強制再生成できる。
2. **ローカルは並列。** `playwright.config.mjs` は workers を CI 以外で CPU 数の半分（上限 4）にし、fullyParallel を true にする。CI は現状の 1 ワーカー・retries 0 を維持する（性能テストは 1 ワーカーでしか意味を持たないので、性能 spec は `serial` 指定で 1 ワーカーに固定する）。
3. **反復用の入口。** `npm run test:spec -- <spec>` で fixture 準備込みの単一 spec 実行ができる。完了時の全件は従来の `npm test`。
4. **結果の同一性。** 並列化の前後で 45 件の成否が同一。共有状態（`.generated` の書き込み、measurements.json）への同時書き込みが無いことを確かめる。
5. **測定。** 変更前後の `npm test` 壁時計（1 回目 = 生成あり、2 回目 = 生成省略）を 3 回ずつ測り、AC-35 の evidence に書く。

### 維持する条件

- CI（`.github/workflows/ci.yml`）の結果と成果物（json レポート、trace、スクリーンショット）は不変。
- `e2e/README.md` の実行手順を同期する。
- `cargo fmt`、clippy、Rust テスト、`bun run check` は無関係だが exit 0 を保つ。

### 完了

- AC-35 を evidence 付きで satisfy。コミットまでで止め、push と公開はマスターの指示待ち。報告は `herdr agent prompt kuroko` で D-33 の形。送信が agent_blocked で失敗したら `herdr agent wait kuroko --until idle` で待ってから再送する。

## 完了確認（kuroko、2026-09-14）

deck の報告をコミット b546773 と c6f7f1e で照合。隔離 worktree: fmt exit 0、clippy 警告 0。npm test（機能 42 件、並列 4 ワーカー）は 1 回目 30.2 秒（fixture 生成あり）、2 回目 17.8 秒、3 回目 17.6 秒、いずれも 42/42。npm run test:all は 54.6 秒で 45/45。CI=1 npm test は testIgnore が空になり全 45 件を 1 ワーカーで実行（ci.yml は `npm test` のまま、成果物も不変）。AC-35 の改訂条件（ローカル既定 30 秒台、test:all は全件で成否同一）を満たす。受領。

閉じた理由: 事実（migrated: complete）

