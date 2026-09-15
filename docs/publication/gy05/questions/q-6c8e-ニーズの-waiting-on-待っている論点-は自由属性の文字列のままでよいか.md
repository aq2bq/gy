# q-6c8e (Q-41) ニーズの waiting-on（待っている論点）は自由属性の文字列のままでよいか、閉じた関係（need waits-on question）にして書き込み時に検証するか

state: closed
decider: lead
options: A: 閉じた関係に足す（Relation を 12 値に。存在しない論点や論点でない ID を書き込み時に拒める。移行で waiting-on 属性を辺に写す）, B: 自由属性のまま（解釈できない語は無視。常に妥当の対象外）
scope: gy05
created: 2026-09-15
  closes d-8637 (D-83) ニーズが待っている論点は閉じた関係 waits-on（need → question）で持ち、書き込み時に検証する。自由属性 waiting-on は移行で辺に写す
## 経緯

N-59 の検収（2026-09-15）で、next の準備判定が自由属性 waiting-on の文字列を ID として読んでいるのを見た。0.4 でも waiting-on は参照属性（REFERENCE_KEYS）で辺ではなかった。書ける経路が gy だけの新しい gy では、辺にすれば D-75 の「不正は書き込み時に拒む」が効く。判断は N-41（移行）の設計時に lead が行い、A なら N-41 と同じニーズで Relation の追加と移行を同梱する（D-77）。推奨は A。

閉じ方: 決定（2026-09-15T03:03:11.919008+00:00）

