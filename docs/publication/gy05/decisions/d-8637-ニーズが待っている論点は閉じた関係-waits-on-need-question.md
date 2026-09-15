# d-8637 (D-83) ニーズが待っている論点は閉じた関係 waits-on（need → question）で持ち、書き込み時に検証する。自由属性 waiting-on は移行で辺に写す

- 種類: decision
- scope: gy05
- created: 2026-09-15
- 別名: D-83

## 関係

- closed-by q-6c8e (Q-41) ニーズの waiting-on（待っている論点）は自由属性の文字列のままでよいか、閉じた関係（need waits-on question）にして書き込み時に検証するか
- spawns n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）
- widened-by d-63f8 waits-on の先には論点に加えて要求も許し、要求が Done か Cancelled になれば待ちが解ける（mark: 組は need → question）

## 成立範囲

Q-41 を lead が 2026-09-15 に決めた（案 A）。Relation を 12 値にし（waits-on / awaited-by、組は need → question）、next の準備判定は自由属性でなく辺を読む。need add と link で張れ、論点が閉じても辺は残す（判定は論点の状態で行う）。移行（N-41）は 0.4 の waiting-on 属性（ID のリスト）を辺に写し、先が論点でない ID は移行の報告に出して辺にしない。非互換の変更なので移行と同じニーズで行う（D-77）。選ばなかった B: 自由属性のまま（存在しない論点を書き込み時に拒めず D-75 に反する）。

## 本文


## Context

## Decision

## Consequences

## 自由属性

- 無し

