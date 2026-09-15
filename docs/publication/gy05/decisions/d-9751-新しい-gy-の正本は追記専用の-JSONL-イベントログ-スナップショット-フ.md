# d-9751 (D-82) 新しい gy の正本は追記専用の JSONL イベントログ + スナップショット（ファイル、依存ゼロ）で持つ

## 関係
- 無し

decision_scope: N-38 以降の store の実体。2026-09-15 マスターが Q-39 の案 A を採用（「A で OK」）。1 トランザクション = ログ 1 行（seq、at、actor、why、source、changes）。原子性は 1 行の追記 + fsync、途中で切れた末尾は開くときに捨てる。競合検出は fs2 の排他ロックと seq の比較。履歴はログそのもので undo は逆変更の行を追記する。移行はログを書き換えず読み手の版を上げる。スナップショットは無くても再生できる派生物。置き場所は $XDG_DATA_HOME/gy/<リポジトリのルートのハッシュ>/ で、リポジトリには gy.toml だけを置く（lead の判断）。選ばなかった案: B SQLite（並行読みの性能で勝るがこの規模では効かず、C の依存が増える）、C 0.4 の YAML 群（履歴がノードに埋め込まれ 95% 問題の原因）。
scope: gy05
created: 2026-09-15
  spawns n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）
  spawns n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）
  spawns n-0cb7 (N-48) 正本の置き場所（XDG のデータディレクトリ + リポジトリのハッシュ）と形式の版 format の読み書き・判定・移行の入口の枠
  spawns n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割）
  spawns n-bd8d (N-49) undo（逆変更の行の追記）と snapshot.json（commit ごとの原子的な書き直し、無くてもログの再生で復元）
  closed-by q-c7b2 (Q-39) 新しい gy の正本の保存形式は何か（N-38 の前提。D-61 の要件から導く）

## Context

## Decision

## Consequences


