# d-63f8 waits-on の先には論点に加えて要求も許し、要求が Done か Cancelled になれば待ちが解ける

## 関係
- widens d-8637 (D-83) ニーズが待っている論点は閉じた関係 waits-on（need → question）で持ち、書き込み時に検証する。自由属性 waiting-on は移行で辺に写す（mark: 組は need → question）

decision_scope: 2026-09-15 Kokopelli の台帳の移行の試行で、N-7 が waiting-on: #5972（要求）を持ち、D-83 の need → question だけでは写せず next に出てしまった。要求の完了を待つニーズは実運用にある形なので、Relation::ALLOWED に (WaitsOn, Need, Requirement) を足し、準備の判定は要求の状態が Done / Cancelled なら解けたとみなす。移行も要求への waiting-on を辺にする。D-83 を widens。
scope: gy05
created: 2026-09-15
  widens d-8637 (D-83) ニーズが待っている論点は閉じた関係 waits-on（need → question）で持ち、書き込み時に検証する。自由属性 waiting-on は移行で辺に写す（mark: 組は need → question）
  spawns n-b6a7 waits-on の先に要求を許す: model の組、derive の準備判定、gy-migrate の waiting-on の写し、テスト

