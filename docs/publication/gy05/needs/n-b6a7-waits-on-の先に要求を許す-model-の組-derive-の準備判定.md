# n-b6a7 waits-on の先に要求を許す: model の組、derive の準備判定、gy-migrate の waiting-on の写し、テスト

state: closed
scope: gy05
created: 2026-09-15
  targets ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  spawned-by d-63f8 waits-on の先には論点に加えて要求も許し、要求が Done か Cancelled になれば待ちが解ける
閉じた理由: 事実（2026-09-15 lead が検収。measure gy-ledger 違反 0（score 140）、gy-migrate は legacy/ の文字列キーのみ。テスト 191 件 0 failed。Kokopelli の写し（74fa00f、356 ノード）で waits-on 17（要求待ち 1 + belongs-to 10 + 元の 6）、skipped 0、next が 0.4 と同じ 6 件（N-1 / N-4 / N-11 / N-15 / N-23 / N-25）。コミット b86c2e9）

