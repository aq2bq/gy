# d-4aa9 (D-80) html_projection の未着手ニーズ 14 件は D-79 により実装せずに閉じ、status を complete にして dropped_by で D-80 を指す

## 関係
- completes d-b64d (D-79) render と今の HTML 投影は最初に消す。伸ばしても到達点に着かず、利用者はマスターだけで移行も要らない。人間向けの投影は第 3 段で publish の読み物から作り直す（mark: 残ニーズは lead が閉じ方を決める）

decision_scope: 対象は html_projection スコープで status が unfiled のニーズ 14 件（N-1〜N-10、N-19〜N-21、N-23）。2026-09-15 lead の判断（D-79 が lead に委ねた）。0.4 の gy にはニーズを中止にする状態が無く、next から外す手段は status: complete だけ（D-28）。したがって status を complete にし、属性 dropped_by: D-80 と dropped_reason: 未実装のまま閉じた。HTML 投影は D-79 で消す、を置いて（closed_by は論点の閉じに予約された属性で node set が拒む）、完了ではなく中止であることを残す。完了済みの 7 件（N-22、N-24〜N-29）と決定・受け入れ条件は変えない。新しい gy への移行（N-41）では、dropped_by を持つ complete のニーズを中止として読み替える。html_projection 以外の 0.4 系スコープ（workflow_records、need_lifecycle、agent_footprint、import_provenance、guard_waiver）の未着手ニーズは別の判断とし、ここでは触らない
scope: html_projection
created: 2026-09-15
  completes d-b64d (D-79) render と今の HTML 投影は最初に消す。伸ばしても到達点に着かず、利用者はマスターだけで移行も要らない。人間向けの投影は第 3 段で publish の読み物から作り直す（mark: 残ニーズは lead が閉じ方を決める）

## Context

## Decision

## Consequences


